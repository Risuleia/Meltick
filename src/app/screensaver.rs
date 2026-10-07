use std::time::Instant;

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
            GWLP_USERDATA, GetCursorPos, GetWindowLongPtrW, MSG, PM_REMOVE, PeekMessageW,
            PostQuitMessage, RegisterClassW, SW_SHOW, SetCursor, SetWindowLongPtrW, ShowWindow,
            TranslateMessage, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEHWHEEL,
            WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_NCDESTROY, WM_RBUTTONDOWN, WM_SETCURSOR,
            WM_SYSKEYDOWN, WM_XBUTTONDOWN, WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    },
    core::Result,
};

use crate::{config::Config, gfx::Gfx, util::enumerate_monitors};

const MOUSE_EXIT_THRESHOLD: i32 = 20;

struct ScreenSaverWindow {
    hwnd: HWND,
    input_origin: POINT,
    gfx: Option<Gfx>,
}

pub fn run_screensaver(config: Config) -> Result<()> {
    let monitors = enumerate_monitors()?;

    if monitors.is_empty() {
        return Err(windows::core::Error::new(
            windows::core::HRESULT(0x80004005u32 as i32),
            "No display monitors found",
        ));
    }

    let module = unsafe { GetModuleHandleW(None)? };

    let hinstance = HINSTANCE(module.0);

    let class_name = windows::core::w!("LiquidGlassScreenSaverWindow");

    let wc = WNDCLASSW {
        hInstance: hinstance,
        lpfnWndProc: Some(screensaver_wndproc),
        lpszClassName: class_name,
        ..Default::default()
    };

    unsafe {
        if RegisterClassW(&wc) == 0 {
            return Err(windows::core::Error::from_thread());
        }
    }

    let mut input_origin = POINT::default();

    unsafe {
        GetCursorPos(&mut input_origin)?;
    }

    let mut windows: Vec<Box<ScreenSaverWindow>> = Vec::with_capacity(monitors.len());

    for rect in monitors {
        let width = rect.right - rect.left;

        let height = rect.bottom - rect.top;

        let mut window = Box::new(ScreenSaverWindow {
            hwnd: HWND::default(),
            input_origin,
            gfx: None,
        });

        let window_ptr = &mut *window as *mut ScreenSaverWindow;

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                windows::core::w!("Liquid Glass Clock"),
                WS_POPUP,
                rect.left,
                rect.top,
                width,
                height,
                None,
                None,
                Some(hinstance),
                Some(window_ptr as *const _),
            )?
        };

        window.hwnd = hwnd;

        window.gfx = Some(Gfx::new(hwnd, width as u32, height as u32, config)?);

        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        windows.push(window);
    }

    let start = Instant::now();

    loop {
        let mut msg = MSG::default();

        unsafe {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    break;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
            break;
        }

        let time = start.elapsed().as_secs_f32();

        for window in &mut windows {
            if let Some(gfx) = window.gfx.as_mut() {
                gfx.render(time)?;
            }
        }
    }

    unsafe {
        for window in &windows {
            DestroyWindow(window.hwnd).ok();
        }
    }

    Ok(())
}

unsafe extern "system" fn screensaver_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = &*(lparam.0 as *const CREATESTRUCTW);

                let window = create.lpCreateParams as *mut ScreenSaverWindow;

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, window as isize);

                (*window).hwnd = hwnd;

                return LRESULT(1);
            }

            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }

            WM_KEYDOWN | WM_SYSKEYDOWN | WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN
            | WM_XBUTTONDOWN | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                PostQuitMessage(0);
                return LRESULT(0);
            }

            WM_MOUSEMOVE => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ScreenSaverWindow;

                if !window_ptr.is_null() {
                    let window = &*window_ptr;

                    let mut point = POINT::default();

                    if GetCursorPos(&mut point).is_ok() {
                        let dx = point.x - window.input_origin.x;

                        let dy = point.y - window.input_origin.y;

                        if dx.abs() > MOUSE_EXIT_THRESHOLD || dy.abs() > MOUSE_EXIT_THRESHOLD {
                            PostQuitMessage(0);
                            return LRESULT(0);
                        }
                    }
                }

                return LRESULT(0);
            }

            WM_SETCURSOR => {
                SetCursor(None);
                return LRESULT(1);
            }

            _ => {}
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
