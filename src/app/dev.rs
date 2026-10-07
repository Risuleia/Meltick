use std::{cell::RefCell, time::Instant};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_ESCAPE, VK_R, VK_SPACE},
            WindowsAndMessaging::{
                CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
                IDC_ARROW, LoadCursorW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage,
                RegisterClassW, TranslateMessage, WINDOW_EX_STYLE, WM_DESTROY, WM_KEYDOWN, WM_QUIT,
                WM_SIZE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    },
    core::{Result, w},
};

use crate::{config::Config, gfx::Gfx};

thread_local! {
    static GFX: RefCell<Option<Gfx>> = const { RefCell::new(None) };
}

pub fn run_dev() -> Result<()> {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class = w!("MeltickWnd");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(dev_wndproc),
            hInstance: hinstance,
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            w!("meltick"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280,
            720,
            None,
            None,
            Some(hinstance),
            None,
        )?;

        let mut rc = RECT::default();
        GetClientRect(hwnd, &mut rc)?;
        let gfx = Gfx::new(
            hwnd,
            (rc.right - rc.left) as u32,
            (rc.bottom - rc.top) as u32,
            Config::default(),
        )?;
        GFX.with(|g| *g.borrow_mut() = Some(gfx));

        let start = Instant::now();
        let mut msg = MSG::default();

        'main: loop {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break 'main;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let t = start.elapsed().as_secs_f32();
            GFX.with(|g| {
                if let Some(gfx) = g.borrow_mut().as_mut() {
                    if let Err(e) = gfx.render(t) {
                        eprintln!("render: {e}");
                    }
                }
            });
        }
    }

    Ok(())
}

extern "system" fn dev_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_SIZE => {
                let w = (lp.0 & 0xFFFF) as u32;
                let h = ((lp.0 >> 16) & 0xFFFF) as u32;

                if w > 0 && h > 0 {
                    GFX.with(|g| {
                        if let Ok(mut b) = g.try_borrow_mut() {
                            if let Some(gfx) = b.as_mut() {
                                if let Err(e) = gfx.resize(w, h) {
                                    eprintln!("resize: {e}");
                                }
                            }
                        }
                    })
                }

                LRESULT(0)
            }

            WM_KEYDOWN => {
                let vk = VIRTUAL_KEY(wp.0 as u16);
                if vk == VK_ESCAPE {
                    PostQuitMessage(0);
                } else if vk == VK_SPACE {
                    GFX.with(|g| {
                        if let Ok(mut b) = g.try_borrow_mut() {
                            if let Some(gfx) = b.as_mut() {
                                gfx.tick();
                            }
                        }
                    });
                } else if vk == VK_R {
                    GFX.with(|g| {
                        if let Ok(mut b) = g.try_borrow_mut() {
                            if let Some(gfx) = b.as_mut() {
                                match gfx.reload_shaders() {
                                    Ok(()) => eprintln!("shaders reloaded"),
                                    Err(e) => eprintln!("shader error: {e}"),
                                }
                            }
                        }
                    });
                }
                LRESULT(0)
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}
