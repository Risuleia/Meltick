use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM}, Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateRoundRectRgn, CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, FillRect, GetMonitorInfoW, InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, PAINTSTRUCT, SRCCOPY, ScreenToClient, SelectObject, SetWindowRgn, UpdateWindow,
        }, System::LibraryLoader::GetModuleHandleW, UI::{
            Controls::{TBS_HORZ, TRACKBAR_CLASS}, Input::KeyboardAndMouse::{ReleaseCapture, SetCapture}, WindowsAndMessaging::{
                BS_PUSHBUTTON, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetCursorPos, GetMessageW, GetWindowLongPtrW, HCURSOR, HMENU, HTCAPTION, IDC_ARROW, LoadCursorW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage, RegisterClassW, SW_SHOW, SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_QUIT, WM_SIZE, WNDCLASSW, WS_CAPTION, WS_CHILD, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_VISIBLE,
            },
        },
    }, core::{Result, w},
};

use crate::app::config::{
    constants::*,
    draw::{self, draw_config_ui},
};
use crate::{
    app::config::components::{Button, ButtonStyle, SegmentedControl, Slider},
    config::Config,
    gfx::Gfx,
};

struct ConfigWindow {
    hwnd: HWND,
    preview_hwnd: HWND,
    preview_gfx: Option<Gfx>,

    original_config: Config,
    working_config: Config,

    format_control: SegmentedControl,
    scale_control: Slider,

    defaults_button: Button,
    cancel_button: Button,
    apply_button: Button,

    hovered_button: Option<ButtonId>,
    pressed_button: Option<ButtonId>,

    dragging_scale: bool,
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

            original_config: config,
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
    let point = GetCursorPos(&mut POINT::default());

    let cursor = if point.is_ok() {
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        point
    } else {
        POINT { x: 0, y: 0 }
    };

    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    let _ = GetMonitorInfoW(monitor, &mut info);

    let work = info.rcWork;

    let x = work.left + ((work.right - work.left) - width) / 2;

    let y = work.top + ((work.bottom - work.top) - height) / 2;

    (x, y)
}

fn point_from_lparam(lparam: LPARAM) -> POINT {
    POINT {
        x: (lparam.0 as i16) as i32,
        y: ((lparam.0 >> 16) as i16) as i32,
    }
}

fn button_at(window: &ConfigWindow, point: POINT) -> Option<ButtonId> {
    if window.defaults_button.contains(point) {
        Some(ButtonId::Defaults)
    } else if window.cancel_button.contains(point) {
        Some(ButtonId::Cancel)
    } else if window.apply_button.contains(point) {
        Some(ButtonId::Apply)
    } else {
        None
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

            WM_LBUTTONDOWN => {
                let point = point_from_lparam(lparam);

                let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

                //
                // BUTTONS
                //
                if let Some(button) = button_at(window, point) {
                    window.pressed_button = Some(button);

                    InvalidateRect(Some(hwnd), None, false);

                    return LRESULT(0);
                }

                //
                // TIME FORMAT
                //
                if let Some(format) = window.format_control.value_at(point) {
                    window.pressed_button = None;

                    if window.working_config.time_format != format {
                        window.working_config.time_format = format;

                        if let Some(gfx) = window.preview_gfx.as_mut() {
                            gfx.set_config(window.working_config);
                        }
                    }

                    InvalidateRect(Some(hwnd), None, false);

                    return LRESULT(0);
                }

                //
                // SCALE
                //
                if window.scale_control.contains(point) {
                    window.pressed_button = None;
                    window.dragging_scale = true;

                    let scale = window.scale_control.scale_from_point(point.x);

                    if window.working_config.scale != scale {
                        window.working_config.scale = scale;

                        if let Some(gfx) = window.preview_gfx.as_mut() {
                            gfx.set_config(window.working_config);
                        }
                    }

                    SetCapture(hwnd);

                    InvalidateRect(Some(hwnd), None, false);

                    return LRESULT(0);
                }

                LRESULT(0)
            }

            WM_MOUSEMOVE => {
                let point = point_from_lparam(lparam);

                let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

                //
                // BUTTON HOVER
                //
                let hovered = button_at(window, point);

                if window.hovered_button != hovered {
                    window.hovered_button = hovered;

                    InvalidateRect(Some(hwnd), None, false);
                }

                //
                // SCALE DRAGGING
                //
                if window.dragging_scale {
                    let scale = window.scale_control.scale_from_point(point.x);

                    if window.working_config.scale != scale {
                        window.working_config.scale = scale;

                        if let Some(gfx) = window.preview_gfx.as_mut() {
                            gfx.set_config(window.working_config);
                        }

                        InvalidateRect(Some(hwnd), None, false);
                    }
                }

                LRESULT(0)
            }

            WM_LBUTTONUP => {
                let point = point_from_lparam(lparam);

                let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

                //
                // SLIDER
                //
                if window.dragging_scale {
                    window.dragging_scale = false;
                    ReleaseCapture();
                }

                //
                // BUTTON
                //
                let pressed = window.pressed_button;
                let released_over = button_at(window, point);

                window.pressed_button = None;

                InvalidateRect(Some(hwnd), None, false);

                //
                // Only activate if the mouse was
                // released over the same button.
                //
                if pressed == released_over {
                    match pressed {
                        Some(ButtonId::Defaults) => {
                            window.working_config = Config::default();

                            if let Some(gfx) = window.preview_gfx.as_mut() {
                                gfx.set_config(window.working_config);
                            }

                            InvalidateRect(Some(hwnd), None, false);
                        }

                        Some(ButtonId::Cancel) => {
                            let _ = DestroyWindow(hwnd);
                        }

                        Some(ButtonId::Apply) => {
                            let mut config = window.working_config;

                            config.validate();

                            match config.save() {
                                Ok(()) => {
                                    window.working_config = config;

                                    let _ = DestroyWindow(hwnd);
                                }

                                Err(error) => {
                                    eprintln!("failed to save configuration: {error}");
                                }
                            }
                        }

                        None => {}
                    }
                }

                LRESULT(0)
            }

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

            WM_NCHITTEST => {
                let window = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const ConfigWindow);

                let point = POINT {
                    x: (lparam.0 as i16) as i32,
                    y: ((lparam.0 >> 16) as i16) as i32,
                };

                let mut client_point = point;

                let _ = ScreenToClient(hwnd, &mut client_point);

                if window.format_control.contains(client_point)
                    || window.scale_control.contains(client_point)
                    || window.defaults_button.contains(client_point)
                    || window.cancel_button.contains(client_point)
                    || window.apply_button.contains(client_point)
                {
                    return DefWindowProcW(hwnd, WM_NCHITTEST, _wparam, lparam);
                }

                LRESULT(HTCAPTION as isize)
            }

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
