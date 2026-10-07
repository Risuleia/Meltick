use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateRoundRectRgn,
            DeleteDC, DeleteObject, EndPaint, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST,
            MONITORINFO, MonitorFromPoint, PAINTSTRUCT, SRCCOPY, SelectObject, SetWindowRgn,
            UpdateWindow,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow,
            DispatchMessageW, GWLP_USERDATA, GetClientRect, GetCursorPos, GetWindowLongPtrW,
            HCURSOR, IDC_ARROW, LoadCursorW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage,
            RegisterClassW, SW_SHOW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
            WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP,
            WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_QUIT, WM_SIZE,
            WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE,
        },
    },
    core::{Result, w},
};

use crate::{
    app::config::{
        components::{Button, ButtonStyle, SegmentedControl, Slider},
        constants::*,
        draw::draw_config_ui,
        input,
    },
    config::Config,
    gfx::Gfx,
};

pub(crate) struct ConfigWindow {
    pub(crate) hwnd: HWND,
    pub(crate) preview_hwnd: HWND,
    pub(crate) preview_gfx: Option<Gfx>,

    pub(crate) working_config: Config,

    pub(crate) format_control: SegmentedControl,
    pub(crate) scale_control: Slider,

    pub(crate) defaults_button: Button,
    pub(crate) cancel_button: Button,
    pub(crate) apply_button: Button,

    pub(crate) hovered_button: Option<ButtonId>,
    pub(crate) pressed_button: Option<ButtonId>,

    pub(crate) dragging_scale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonId {
    Defaults,
    Cancel,
    Apply,
}

pub fn run_config(parent: Option<HWND>, config: Config) -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?.into();

        let config_class = w!("MeltickConfigWindow");
        let preview_class = w!("MeltickConfigPreview");

        let config_wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(config_wnd_proc),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: config_class,
            ..Default::default()
        };

        let _ = RegisterClassW(&config_wc);

        let preview_wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(config_preview_wnd_proc),
            hInstance: instance,
            hCursor: HCURSOR::default(),
            lpszClassName: preview_class,
            ..Default::default()
        };

        let _ = RegisterClassW(&preview_wc);

        let mut window = Box::new(ConfigWindow {
            hwnd: HWND::default(),
            preview_hwnd: HWND::default(),
            preview_gfx: None,

            working_config: config,

            format_control: SegmentedControl::new(FORMAT_X, FORMAT_Y, FORMAT_W, FORMAT_H),

            scale_control: Slider::new(SCALE_X, SCALE_Y, SCALE_W, SCALE_H),

            defaults_button: Button::new(
                DEFAULTS_X,
                DEFAULTS_Y,
                DEFAULTS_W,
                DEFAULTS_H,
                "Defaults",
                ButtonStyle::Secondary,
            ),

            cancel_button: Button::new(
                CANCEL_X,
                ACTION_Y,
                ACTION_W,
                ACTION_H,
                "Cancel",
                ButtonStyle::Secondary,
            ),

            apply_button: Button::new(
                APPLY_X,
                ACTION_Y,
                ACTION_W,
                ACTION_H,
                "Apply",
                ButtonStyle::Primary,
            ),

            hovered_button: None,
            pressed_button: None,

            dragging_scale: false,
        });

        let window_ptr = &mut *window as *mut ConfigWindow;

        let (x, y) = centered_window_position(CONFIG_WIDTH, CONFIG_HEIGHT);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            config_class,
            w!("Meltic"),
            WS_POPUP,
            x,
            y,
            CONFIG_WIDTH,
            CONFIG_HEIGHT,
            parent,
            None,
            Some(instance),
            Some(window_ptr as *const _),
        )?;

        if hwnd.0.is_null() {
            return Err(windows::core::Error::from_thread());
        }

        let region = CreateRoundRectRgn(
            0,
            0,
            CONFIG_WIDTH + 1,
            CONFIG_HEIGHT + 1,
            WINDOW_RADIUS * 2,
            WINDOW_RADIUS * 2,
        );

        SetWindowRgn(hwnd, Some(region), true);

        window.hwnd = hwnd;

        let preview_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            preview_class,
            w!(""),
            WS_CHILD | WS_VISIBLE,
            PREVIEW_MARGIN_X,
            PREVIEW_TOP,
            CONFIG_WIDTH - PREVIEW_MARGIN_X * 2,
            PREVIEW_HEIGHT,
            Some(hwnd),
            None,
            Some(instance),
            Some(window_ptr as *const _),
        )?;

        if preview_hwnd.0.is_null() {
            return Err(windows::core::Error::from_thread());
        }

        window.preview_hwnd = preview_hwnd;

        window.preview_gfx = Some(Gfx::new(
            preview_hwnd,
            (CONFIG_WIDTH - PREVIEW_MARGIN_X * 2) as u32,
            PREVIEW_HEIGHT as u32,
            window.working_config,
        )?);

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let start = std::time::Instant::now();
        let mut msg = MSG::default();

        loop {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if msg.message == WM_QUIT {
                break;
            }

            let time = start.elapsed().as_secs_f32();

            if let Some(gfx) = window.preview_gfx.as_mut() {
                gfx.render(time)?;
            }
        }

        Ok(())
    }
}

unsafe fn centered_window_position(width: i32, height: i32) -> (i32, i32) {
    unsafe {
        let point = GetCursorPos(&mut POINT::default());

        let cursor = if point.is_ok() {
            let mut point = POINT::default();
            let _ = GetCursorPos(&mut point);
            point
        } else {
            POINT { x: 0, y: 0 }
        };

        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);

        let mut info =
            MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };

        let _ = GetMonitorInfoW(monitor, &mut info);

        let work = info.rcWork;

        let x = work.left + ((work.right - work.left) - width) / 2;

        let y = work.top + ((work.bottom - work.top) - height) / 2;

        (x, y)
    }
}

unsafe extern "system" fn config_wnd_proc(
    hwnd: HWND,
    msg: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = &*(lparam.0 as *const CREATESTRUCTW);

                let window = create.lpCreateParams as *mut ConfigWindow;

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, window as isize);

                (*window).hwnd = hwnd;

                LRESULT(1)
            }

            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }

            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

                DefWindowProcW(hwnd, msg, _wparam, lparam)
            }

            WM_LBUTTONDOWN => input::handle_lbutton_down(hwnd, lparam),

            WM_LBUTTONUP => input::handle_lbutton_up(hwnd, lparam),

            WM_MOUSEMOVE => input::handle_mousemove(hwnd, lparam),

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();

                let hdc = BeginPaint(hwnd, &mut ps);

                let mut client = RECT::default();

                let _ = GetClientRect(hwnd, &mut client);

                let width = client.right - client.left;
                let height = client.bottom - client.top;

                if width > 0 && height > 0 {
                    let memory_dc = CreateCompatibleDC(Some(hdc));

                    if !memory_dc.is_invalid() {
                        let bitmap = CreateCompatibleBitmap(hdc, width, height);

                        if !bitmap.is_invalid() {
                            let old_bitmap = SelectObject(memory_dc, bitmap.into());

                            let window =
                                &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

                            draw_config_ui(
                                memory_dc,
                                client,
                                &window.working_config,
                                &window.format_control,
                                &window.scale_control,
                                &window.defaults_button,
                                &window.cancel_button,
                                &window.apply_button,
                                window.hovered_button,
                                window.pressed_button,
                            );

                            let _ =
                                BitBlt(hdc, 0, 0, width, height, Some(memory_dc), 0, 0, SRCCOPY);

                            SelectObject(memory_dc, old_bitmap);

                            let _ = DeleteObject(bitmap.into());
                        }

                        let _ = DeleteDC(memory_dc);
                    }
                }

                let _ = EndPaint(hwnd, &ps);

                LRESULT(0)
            }

            WM_NCHITTEST => input::handle_nchittest(hwnd, lparam, _wparam),

            WM_ERASEBKGND => LRESULT(1),

            _ => DefWindowProcW(hwnd, msg, _wparam, lparam),
        }
    }
}

unsafe extern "system" fn config_preview_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = &*(lparam.0 as *const CREATESTRUCTW);

                let window = create.lpCreateParams as *mut ConfigWindow;

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, window as isize);

                (*window).preview_hwnd = hwnd;

                LRESULT(1)
            }

            WM_SIZE => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow;

                if !window_ptr.is_null() {
                    let window = &mut *window_ptr;

                    let width = (lparam.0 & 0xffff) as u32;
                    let height = ((lparam.0 >> 16) & 0xffff) as u32;

                    if width > 0 && height > 0 {
                        if let Some(gfx) = window.preview_gfx.as_mut() {
                            let _ = gfx.resize(width, height);
                        }
                    }
                }

                LRESULT(0)
            }

            WM_ERASEBKGND => LRESULT(1),

            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

                DefWindowProcW(hwnd, msg, wparam, lparam)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
