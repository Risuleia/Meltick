use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::Gdi::{InvalidateRect, ScreenToClient},
    UI::{
        Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
        WindowsAndMessaging::{
            DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetWindowLongPtrW, HTCAPTION,
            WM_NCHITTEST,
        },
    },
};

use crate::{
    app::config::window::{ButtonId, ConfigWindow},
    config::Config,
};

pub unsafe fn handle_lbutton_down(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    unsafe {
        let point = point_from_lparam(lparam);

        let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

        // BUTTONS
        if let Some(button) = button_at(window, point) {
            window.pressed_button = Some(button);

            let _ = InvalidateRect(Some(hwnd), None, false);

            return LRESULT(0);
        }

        // TIME FORMAT
        if let Some(format) = window.format_control.value_at(point) {
            window.pressed_button = None;

            if window.working_config.time_format != format {
                window.working_config.time_format = format;

                if let Some(gfx) = window.preview_gfx.as_mut() {
                    gfx.set_config(window.working_config);
                }
            }

            let _ = InvalidateRect(Some(hwnd), None, false);

            return LRESULT(0);
        }

        // SCALE
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

            let _ = InvalidateRect(Some(hwnd), None, false);

            return LRESULT(0);
        }
    }

    LRESULT(0)
}

pub unsafe fn handle_lbutton_up(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    unsafe {
        let point = point_from_lparam(lparam);

        let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

        // SLIDER
        if window.dragging_scale {
            window.dragging_scale = false;
            let _ = ReleaseCapture();
        }

        // BUTTON
        let pressed = window.pressed_button;
        let released_over = button_at(window, point);

        window.pressed_button = None;

        let _ = InvalidateRect(Some(hwnd), None, false);

        // Only activate if the mouse was released over the same button.
        if pressed == released_over {
            match pressed {
                Some(ButtonId::Defaults) => {
                    window.working_config = Config::default();

                    if let Some(gfx) = window.preview_gfx.as_mut() {
                        gfx.set_config(window.working_config);
                    }

                    let _ = InvalidateRect(Some(hwnd), None, false);
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
    }

    LRESULT(0)
}

pub unsafe fn handle_mousemove(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    unsafe {
        let point = point_from_lparam(lparam);

        let window = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ConfigWindow);

        // BUTTON HOVER
        let hovered = button_at(window, point);

        if window.hovered_button != hovered {
            window.hovered_button = hovered;

            let _ = InvalidateRect(Some(hwnd), None, false);
        }

        // SCALE DRAGGING
        if window.dragging_scale {
            let scale = window.scale_control.scale_from_point(point.x);

            if window.working_config.scale != scale {
                window.working_config.scale = scale;

                if let Some(gfx) = window.preview_gfx.as_mut() {
                    gfx.set_config(window.working_config);
                }

                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
    }

    LRESULT(0)
}

pub unsafe fn handle_nchittest(hwnd: HWND, lparam: LPARAM, wparam: WPARAM) -> LRESULT {
    unsafe {
        let window = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const ConfigWindow);

        let point = POINT { x: (lparam.0 as i16) as i32, y: ((lparam.0 >> 16) as i16) as i32 };

        let mut client_point = point;

        let _ = ScreenToClient(hwnd, &mut client_point);

        if window.format_control.contains(client_point)
            || window.scale_control.contains(client_point)
            || window.defaults_button.contains(client_point)
            || window.cancel_button.contains(client_point)
            || window.apply_button.contains(client_point)
        {
            return DefWindowProcW(hwnd, WM_NCHITTEST, wparam, lparam);
        }

        LRESULT(HTCAPTION as isize)
    }
}

fn point_from_lparam(lparam: LPARAM) -> POINT {
    POINT { x: (lparam.0 as i16) as i32, y: ((lparam.0 >> 16) as i16) as i32 }
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
