use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW,
            DispatchMessageW, GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, HCURSOR, MSG,
            PM_REMOVE, PeekMessageW, RegisterClassW, SetWindowLongPtrW, TranslateMessage,
            WINDOW_EX_STYLE, WM_ERASEBKGND, WM_NCCREATE, WM_NCDESTROY, WM_QUIT, WM_SIZE, WNDCLASSW,
            WS_CHILD, WS_VISIBLE,
        },
    },
    core::{Result, w},
};

use crate::{config::Config, gfx::Gfx};

struct PreviewWindow {
    hwnd: HWND,
    gfx: Option<Gfx>,
}

pub fn run_preview(parent: HWND, config: Config) -> Result<()> {
    unsafe {
        let mut rect = RECT::default();

        GetClientRect(parent, &mut rect)?;

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let class_name = w!("MelticPreviewWindow");

        let instance = GetModuleHandleW(None)?.into();

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(preview_wndproc),
            hInstance: instance,
            hCursor: HCURSOR::default(),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let mut window = Box::new(PreviewWindow { hwnd: HWND::default(), gfx: None });

        let window_ptr = &mut *window as *mut PreviewWindow;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Meltick Preview"),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            width,
            height,
            Some(parent),
            None,
            Some(instance),
            Some(window_ptr as *const _),
        );

        if hwnd.clone()?.0.is_null() {
            return Err(windows::core::Error::from_thread());
        }

        window.hwnd = hwnd.clone()?;
        window.gfx = Some(Gfx::new(hwnd?, width as u32, height as u32, config)?);

        let start = std::time::Instant::now();

        loop {
            let mut msg = MSG::default();

            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return Ok(());
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let t = start.elapsed().as_secs_f32();

            if let Some(gfx) = window.gfx.as_mut() {
                gfx.render(t)?;
            }
        }
    }
}

unsafe extern "system" fn preview_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = &*(lparam.0 as *const CREATESTRUCTW);

                let window = create.lpCreateParams as *mut PreviewWindow;

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, window as isize);

                (*window).hwnd = hwnd;

                LRESULT(1)
            }

            WM_SIZE => {
                let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PreviewWindow;

                if !window_ptr.is_null() {
                    let window = &mut *window_ptr;

                    let width = (lparam.0 & 0xffff) as u32;
                    let height = ((lparam.0 >> 16) & 0xffff) as u32;

                    if width > 0 && height > 0 {
                        if let Some(gfx) = window.gfx.as_mut() {
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
