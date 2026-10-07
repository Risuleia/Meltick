use std::ffi::c_void;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct3D::*,
            Direct3D11::*,
            Dxgi::{Common::*, *},
        },
    },
    core::*,
};

use crate::{
    clock::ClockDigits,
    config::{Config, TimeFormat},
    spring::Spring,
};

const VS_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fullscreen_vs.cso"));
const BG_PS_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bg_ps.cso"));
const GLASS_PS_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/glass_ps.cso"));
const DIGITS_PS_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/digits_ps.cso"));

const IMPULSE_TOP: f32 = 1400.0; // px/s kick on the leading (top) edge
const IMPULSE_BOT: f32 = 700.0; // px/s kick on the trailing (bottom) edge
const TOP_K: f32 = 220.0; // stiffness: higher = snappier
const TOP_C: f32 = 13.0; // damping: lower = more wobble
const BOT_K: f32 = 140.0; // softer spring => bottom lags behind the top
const BOT_C: f32 = 11.0;

const CARD_SIZE_FRAC: f32 = 0.62;
const CARD_RADIUS_FRAC: f32 = 0.12;
const CARD_GAP_FRAC: f32 = 0.083;

const ROLL_GLASS_IMPULSE: f32 = 500.0;
const DIGIT_ROLL_SPEED: f32 = 4.0;

const DRIFT_AMPLITUDE: f32 = 8.0;
const DRIFT_PERIOD: f32 = 300.0;
const DRIFT_Y_RATIO: f32 = 0.73;

#[repr(C)]
struct BgParams {
    resolution: [f32; 2],
    time: f32,
    _pad: f32,

    card0: [f32; 4], // center_x, center_y, width, height
    card1: [f32; 4], // center_x, center_y, width, height

    roll0_a: [f32; 4],
    roll0_b: [f32; 4],

    roll1_a: [f32; 4],
    roll1_b: [f32; 4],

    ampm_visible: f32,
    is_pm: f32,
    _ampm_pad: [f32; 2],
}

#[repr(C)]
struct GlassParams {
    blur_strength: f32,
    refraction: f32,
    glow_power: f32,
    shadow_power: f32,
    radius: f32,
    size: [f32; 2],
    _pad0: f32,
    center: [f32; 2],
    resolution: [f32; 2],
}

struct CardSprings {
    top: Spring,
    bot: Spring,

    digit_a: DigitRoll,
    digit_b: DigitRoll,
}

impl CardSprings {
    fn new(digit_a: u8, digit_b: u8) -> Self {
        Self {
            top: Spring::new(),
            bot: Spring::new(),

            digit_a: DigitRoll::new(digit_a),
            digit_b: DigitRoll::new(digit_b),
        }
    }

    fn step(&mut self, dt: f32) {
        self.top.step(dt, TOP_K, TOP_C);
        self.bot.step(dt, BOT_K, BOT_C);

        self.digit_a.step(dt);
        self.digit_b.step(dt);
    }

    fn tick(&mut self) {
        self.top.v += IMPULSE_TOP + ROLL_GLASS_IMPULSE;
        self.bot.v += IMPULSE_BOT;
    }
}

struct DigitRoll {
    current: u8,
    next: u8,
    progress: f32,
    active: bool,
}

impl DigitRoll {
    fn new(digit: u8) -> Self {
        Self { current: digit, next: digit, progress: 0.0, active: false }
    }

    fn trigger(&mut self, next: u8) {
        if next == self.current || self.active {
            return;
        }

        self.next = next;
        self.progress = 0.0;
        self.active = true;
    }

    fn step(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        // temp animation speed
        self.progress += dt * DIGIT_ROLL_SPEED;

        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.current = self.next;
            self.active = false;
        }
    }
}

pub struct Gfx {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    swapchain: IDXGISwapChain1,

    backbuffer: Option<ID3D11Texture2D>,
    bb_rtv: Option<ID3D11RenderTargetView>,

    bg_tex: ID3D11Texture2D,
    bg_rtv: ID3D11RenderTargetView,
    bg_srv: ID3D11ShaderResourceView,

    digit_atlas: ID3D11ShaderResourceView,
    digits_blend: ID3D11BlendState,
    ampm_atlas: ID3D11ShaderResourceView,

    vs: ID3D11VertexShader,
    bg_ps: ID3D11PixelShader,
    glass_ps: ID3D11PixelShader,
    digits_ps: ID3D11PixelShader,

    bg_cb: ID3D11Buffer,
    glass_cb: ID3D11Buffer,

    sampler: ID3D11SamplerState,

    cards: [CardSprings; 2],
    clock: ClockDigits,
    last_t: f32,

    config: Config,

    w: u32,
    h: u32,
}

fn make_digit_atlas(device: &ID3D11Device) -> Result<ID3D11ShaderResourceView> {
    const ATLAS_W: u32 = 7200;
    const ATLAS_H: u32 = 1792;

    let pixels: &[u8] = include_bytes!("../assets/bebas_digits_sdf_4x.r8");

    assert_eq!(pixels.len(), (ATLAS_W * ATLAS_H) as usize);

    let desc = D3D11_TEXTURE2D_DESC {
        Width: ATLAS_W,
        Height: ATLAS_H,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
    };

    let init = D3D11_SUBRESOURCE_DATA {
        pSysMem: pixels.as_ptr() as *const c_void,
        SysMemPitch: ATLAS_W,
        SysMemSlicePitch: 0,
    };

    unsafe {
        let mut tex = None;

        device.CreateTexture2D(&desc, Some(&init), Some(&mut tex))?;

        let tex = tex.unwrap();

        let mut srv = None;

        device.CreateShaderResourceView(&tex, None, Some(&mut srv))?;

        Ok(srv.unwrap())
    }
}

fn make_ampm_atlas(device: &ID3D11Device) -> Result<ID3D11ShaderResourceView> {
    const ATLAS_W: u32 = 2880;
    const ATLAS_H: u32 = 1792;

    let pixels: &[u8] = include_bytes!("../assets/bebas_ampm_sdf_4x.r8");

    assert_eq!(pixels.len(), (ATLAS_W * ATLAS_H) as usize);

    let desc = D3D11_TEXTURE2D_DESC {
        Width: ATLAS_W,
        Height: ATLAS_H,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
    };

    let init = D3D11_SUBRESOURCE_DATA {
        pSysMem: pixels.as_ptr() as *const c_void,
        SysMemPitch: ATLAS_W,
        SysMemSlicePitch: 0,
    };

    unsafe {
        let mut tex = None;

        device.CreateTexture2D(&desc, Some(&init), Some(&mut tex))?;

        let tex = tex.unwrap();

        let mut srv = None;

        device.CreateShaderResourceView(&tex, None, Some(&mut srv))?;

        Ok(srv.unwrap())
    }
}

fn make_shaders(
    device: &ID3D11Device,
) -> Result<(ID3D11VertexShader, ID3D11PixelShader, ID3D11PixelShader, ID3D11PixelShader)> {
    unsafe {
        let mut vs = None;
        let mut bg = None;
        let mut gl = None;
        let mut digits = None;

        device.CreateVertexShader(VS_BYTECODE, None, Some(&mut vs))?;
        device.CreatePixelShader(BG_PS_BYTECODE, None, Some(&mut bg))?;
        device.CreatePixelShader(GLASS_PS_BYTECODE, None, Some(&mut gl))?;
        device.CreatePixelShader(DIGITS_PS_BYTECODE, None, Some(&mut digits))?;

        Ok((vs.unwrap(), bg.unwrap(), gl.unwrap(), digits.unwrap()))
    }
}

fn make_bg_target(
    device: &ID3D11Device,
    w: u32,
    h: u32,
) -> Result<(ID3D11Texture2D, ID3D11RenderTargetView, ID3D11ShaderResourceView)> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        ..Default::default()
    };
    unsafe {
        let mut tex = None;
        device.CreateTexture2D(&desc, None, Some(&mut tex))?;
        let tex = tex.unwrap();
        let mut rtv = None;
        let mut srv = None;
        device.CreateRenderTargetView(&tex, None, Some(&mut rtv))?;
        device.CreateShaderResourceView(&tex, None, Some(&mut srv))?;
        Ok((tex, rtv.unwrap(), srv.unwrap()))
    }
}

fn make_digits_blend_state(device: &ID3D11Device) -> Result<ID3D11BlendState> {
    let render_target = D3D11_RENDER_TARGET_BLEND_DESC {
        BlendEnable: true.into(),

        SrcBlend: D3D11_BLEND_SRC_ALPHA,
        DestBlend: D3D11_BLEND_INV_SRC_ALPHA,

        BlendOp: D3D11_BLEND_OP_ADD,

        SrcBlendAlpha: D3D11_BLEND_ONE,
        DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,

        BlendOpAlpha: D3D11_BLEND_OP_ADD,

        RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
    };

    let desc = D3D11_BLEND_DESC {
        AlphaToCoverageEnable: false.into(),
        IndependentBlendEnable: false.into(),
        RenderTarget: [
            render_target,
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
        ],
    };

    unsafe {
        let mut state = None;

        device.CreateBlendState(&desc, Some(&mut state))?;

        Ok(state.unwrap())
    }
}

fn make_cbuffer(device: &ID3D11Device, size: usize) -> Result<ID3D11Buffer> {
    let desc = D3D11_BUFFER_DESC {
        ByteWidth: size as u32,
        Usage: D3D11_USAGE_DYNAMIC,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
        ..Default::default()
    };
    let mut buf = None;
    unsafe { device.CreateBuffer(&desc, None, Some(&mut buf))? };
    Ok(buf.unwrap())
}

unsafe fn write_cbuffer<T>(ctx: &ID3D11DeviceContext, buf: &ID3D11Buffer, value: T) -> Result<()> {
    let mut m = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
        ctx.Map(buf, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut m))?;
        std::ptr::write(m.pData as *mut T, value);
        ctx.Unmap(buf, 0);
    }
    Ok(())
}

impl Gfx {
    pub fn new(hwnd: HWND, w: u32, h: u32, config: Config) -> Result<Self> {
        unsafe {
            let mut device = None;
            let mut ctx = None;
            D3D11CreateDevice(
                None::<&IDXGIAdapter>,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut ctx),
            )?;
            let device = device.unwrap();
            let ctx = ctx.unwrap();

            let dxgi: IDXGIDevice = device.cast()?;
            let factory: IDXGIFactory2 = dxgi.GetAdapter()?.GetParent()?;
            let sc_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: w,
                Height: h,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };
            let swapchain = factory.CreateSwapChainForHwnd(&device, hwnd, &sc_desc, None, None)?;

            let backbuffer: ID3D11Texture2D = swapchain.GetBuffer(0)?;
            let mut bb_rtv = None;
            device.CreateRenderTargetView(&backbuffer, None, Some(&mut bb_rtv))?;

            let (bg_tex, bg_rtv, bg_srv) = make_bg_target(&device, w, h)?;
            let (vs, bg_ps, glass_ps, digits_ps) = make_shaders(&device)?;
            let digit_atlas = make_digit_atlas(&device)?;
            let digits_blend = make_digits_blend_state(&device)?;
            let ampm_atlas = make_ampm_atlas(&device)?;
            let bg_cb = make_cbuffer(&device, std::mem::size_of::<BgParams>())?;
            let glass_cb = make_cbuffer(&device, std::mem::size_of::<GlassParams>())?;

            let samp_desc = D3D11_SAMPLER_DESC {
                Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                MaxLOD: f32::MAX,
                ..Default::default()
            };
            let mut sampler = None;
            device.CreateSamplerState(&samp_desc, Some(&mut sampler))?;

            let clock = ClockDigits::now(config.time_format);

            Ok(Self {
                device,
                ctx,
                swapchain,
                backbuffer: Some(backbuffer),
                bb_rtv,
                bg_tex,
                bg_rtv,
                bg_srv,
                digit_atlas,
                digits_blend,
                ampm_atlas,
                vs,
                bg_ps,
                glass_ps,
                digits_ps,
                bg_cb,
                glass_cb,
                sampler: sampler.unwrap(),
                cards: [
                    CardSprings::new(clock.digits[0], clock.digits[1]),
                    CardSprings::new(clock.digits[2], clock.digits[3]),
                ],
                clock,
                last_t: 0.0,
                config,
                w,
                h,
            })
        }
    }

    /// Debug tick: content moves up, top edge leads, bottom edge lags.
    pub fn tick(&mut self) {
        for card in &mut self.cards {
            card.tick();
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;

        let new_clock = ClockDigits::now(self.config.time_format);

        self.clock = new_clock;

        self.cards[0].digit_a.current = new_clock.digits[0];
        self.cards[0].digit_a.next = new_clock.digits[0];
        self.cards[0].digit_a.progress = 0.0;
        self.cards[0].digit_a.active = false;

        self.cards[0].digit_b.current = new_clock.digits[1];
        self.cards[0].digit_b.next = new_clock.digits[1];
        self.cards[0].digit_b.progress = 0.0;
        self.cards[0].digit_b.active = false;

        self.cards[1].digit_a.current = new_clock.digits[2];
        self.cards[1].digit_a.next = new_clock.digits[2];
        self.cards[1].digit_a.progress = 0.0;
        self.cards[1].digit_a.active = false;

        self.cards[1].digit_b.current = new_clock.digits[3];
        self.cards[1].digit_b.next = new_clock.digits[3];
        self.cards[1].digit_b.progress = 0.0;
        self.cards[1].digit_b.active = false;
    }

    pub fn resize(&mut self, w: u32, h: u32) -> Result<()> {
        unsafe {
            // release every reference to the swapchain buffer before ResizeBuffers
            self.ctx.OMSetRenderTargets(None, None);
            self.ctx.ClearState();
            self.bb_rtv = None;
            self.backbuffer = None;

            self.swapchain.ResizeBuffers(0, w, h, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))?;

            let bb: ID3D11Texture2D = self.swapchain.GetBuffer(0)?;
            let mut rtv = None;
            self.device.CreateRenderTargetView(&bb, None, Some(&mut rtv))?;
            self.backbuffer = Some(bb);
            self.bb_rtv = rtv;

            let (t, r, s) = make_bg_target(&self.device, w, h)?;
            self.bg_tex = t;
            self.bg_rtv = r;
            self.bg_srv = s;
            self.w = w;
            self.h = h;
        }
        Ok(())
    }

    fn update_clock(&mut self) {
        let new_clock = ClockDigits::now(self.config.time_format);

        let mut card0_changed = false;
        let mut card1_changed = false;

        for i in 0..4 {
            if new_clock.digits[i] == self.clock.digits[i] {
                continue;
            }

            let next = new_clock.digits[i];

            match i {
                0 => {
                    self.cards[0].digit_a.trigger(next);
                    card0_changed = true;
                }

                1 => {
                    self.cards[0].digit_b.trigger(next);
                    card0_changed = true;
                }

                2 => {
                    self.cards[1].digit_a.trigger(next);
                    card1_changed = true;
                }

                3 => {
                    self.cards[1].digit_b.trigger(next);
                    card1_changed = true;
                }

                _ => unreachable!(),
            }
        }

        if card0_changed {
            self.cards[0].tick();
        }

        if card1_changed {
            self.cards[1].tick();
        }

        self.clock = new_clock;
    }

    pub fn render(&mut self, time: f32) -> Result<()> {
        let dt = (time - self.last_t).clamp(0.0, 0.05);
        self.last_t = time;

        self.update_clock();

        for card in &mut self.cards {
            card.step(dt);
        }

        let (sw, sh) = (self.w as f32, self.h as f32);

        let drift_phase = time * std::f32::consts::TAU / DRIFT_PERIOD;

        let drift_x = drift_phase.sin() * DRIFT_AMPLITUDE;
        let drift_y = (drift_phase * DRIFT_Y_RATIO).sin() * DRIFT_AMPLITUDE;

        let scale = self.config.scale.clamp(Config::MIN_SCALE, Config::MAX_SCALE);

        let base_card_size = (sw.min(sh) * CARD_SIZE_FRAC).max(120.0);

        let card_size = base_card_size * scale;

        let card_w = card_size;
        let base_h = card_size;
        let base_r = card_size * CARD_RADIUS_FRAC;

        let gap = card_size * CARD_GAP_FRAC;

        let group_w = card_w * 2.0 + gap;
        let group_left = (sw - group_w) * 0.5 + drift_x;

        let mut card_geometry = [[0.0f32; 4]; 2];

        for (index, card) in self.cards.iter().enumerate() {
            let center_x = group_left + card_w * 0.5 + index as f32 * (card_w + gap);

            let top_y = sh * 0.5 - base_h * 0.5 + drift_y - card.top.x;
            let bot_y = sh * 0.5 + base_h * 0.5 + drift_y - card.bot.x;

            let h = (bot_y - top_y).max(40.0);

            let cy = (top_y + bot_y) * 0.5;

            let w = card_w * (base_h / h).sqrt();

            card_geometry[index] = [center_x, cy, w, h];
        }

        unsafe {
            write_cbuffer(
                &self.ctx,
                &self.bg_cb,
                BgParams {
                    resolution: [sw, sh],
                    time,
                    _pad: 0.0,

                    card0: card_geometry[0],
                    card1: card_geometry[1],

                    roll0_a: [
                        self.cards[0].digit_a.current as f32,
                        self.cards[0].digit_a.next as f32,
                        self.cards[0].digit_a.progress,
                        if self.cards[0].digit_a.active { 1.0 } else { 0.0 },
                    ],

                    roll0_b: [
                        self.cards[0].digit_b.current as f32,
                        self.cards[0].digit_b.next as f32,
                        self.cards[0].digit_b.progress,
                        if self.cards[0].digit_b.active { 1.0 } else { 0.0 },
                    ],

                    roll1_a: [
                        self.cards[1].digit_a.current as f32,
                        self.cards[1].digit_a.next as f32,
                        self.cards[1].digit_a.progress,
                        if self.cards[1].digit_a.active { 1.0 } else { 0.0 },
                    ],

                    roll1_b: [
                        self.cards[1].digit_b.current as f32,
                        self.cards[1].digit_b.next as f32,
                        self.cards[1].digit_b.progress,
                        if self.cards[1].digit_b.active { 1.0 } else { 0.0 },
                    ],

                    ampm_visible: if self.config.time_format == TimeFormat::H12 {
                        1.0
                    } else {
                        0.0
                    },

                    is_pm: if self.clock.is_pm { 1.0 } else { 0.0 },

                    _ampm_pad: [0.0, 0.0],
                },
            )?;

            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: sw,
                Height: sh,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };

            self.ctx.RSSetViewports(Some(&[viewport]));

            self.ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            self.ctx.VSSetShader(&self.vs, None);

            // PASS 1
            // Aurora + rolling digits -> bg_tex
            self.ctx.OMSetRenderTargets(Some(&[Some(self.bg_rtv.clone())]), None);

            self.ctx.PSSetShader(&self.bg_ps, None);

            self.ctx.PSSetConstantBuffers(0, Some(&[Some(self.bg_cb.clone())]));

            self.ctx.PSSetShaderResources(
                1,
                Some(&[Some(self.digit_atlas.clone()), Some(self.ampm_atlas.clone())]),
            );

            self.ctx.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));

            self.ctx.Draw(3, 0);

            // PASS 2
            // bg_tex -> backbuffer
            let backbuffer = self.backbuffer.as_ref().unwrap();

            self.ctx.CopyResource(backbuffer, &self.bg_tex);

            self.ctx.OMSetRenderTargets(Some(&[self.bb_rtv.clone()]), None);

            // PASS 3
            // Glass cards
            self.ctx.OMSetBlendState(
                Some(&self.digits_blend),
                Some(&[0.0, 0.0, 0.0, 0.0]),
                u32::MAX,
            );

            self.ctx.PSSetShader(&self.glass_ps, None);

            self.ctx.PSSetShaderResources(0, Some(&[Some(self.bg_srv.clone())]));

            self.ctx.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));

            for (index, _card) in self.cards.iter().enumerate() {
                let center_x = card_geometry[index][0];
                let cy = card_geometry[index][1];
                let w = card_geometry[index][2];
                let h = card_geometry[index][3];

                let r = (base_r * (h / base_h).sqrt()).min(w.min(h) * 0.5);

                write_cbuffer(
                    &self.ctx,
                    &self.glass_cb,
                    GlassParams {
                        blur_strength: 3.0,
                        refraction: 0.6,
                        glow_power: 1.0,
                        shadow_power: 1.0,

                        radius: r,

                        size: [w, h],

                        _pad0: 0.0,

                        center: [center_x, cy],

                        resolution: [sw, sh],
                    },
                )?;

                self.ctx.PSSetConstantBuffers(0, Some(&[Some(self.glass_cb.clone())]));

                self.ctx.Draw(3, 0);
            }

            self.ctx.OMSetBlendState(None, Some(&[0.0, 0.0, 0.0, 0.0]), u32::MAX);

            self.ctx.PSSetShaderResources(0, Some(&[None, None]));

            // PASS 4
            // Crisp rolling digits over glass
            self.ctx.OMSetBlendState(
                Some(&self.digits_blend),
                Some(&[0.0, 0.0, 0.0, 0.0]),
                u32::MAX,
            );

            self.ctx.PSSetShader(&self.digits_ps, None);

            self.ctx.PSSetShaderResources(
                1,
                Some(&[Some(self.digit_atlas.clone()), Some(self.ampm_atlas.clone())]),
            );

            self.ctx.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));

            self.ctx.PSSetConstantBuffers(0, Some(&[Some(self.bg_cb.clone())]));

            self.ctx.Draw(3, 0);

            self.ctx.PSSetShaderResources(1, Some(&[None, None]));

            self.ctx.OMSetBlendState(None, Some(&[0.0, 0.0, 0.0, 0.0]), u32::MAX);

            self.swapchain.Present(1, DXGI_PRESENT(0)).ok()
        }
    }
}
