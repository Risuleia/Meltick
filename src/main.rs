#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clock;
mod config;
mod gfx;
mod spring;
mod util;

fn main() {
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }

    let config = config::Config::load();

    match app::parse_screen_saver_mode() {
        app::ScreenSaverMode::Dev => {
            if let Err(e) = app::run_dev() {
                eprintln!("fatal: {e}");
            }
        }

        app::ScreenSaverMode::ScreenSaver => {
            if let Err(e) = app::run_screensaver(config) {
                eprintln!("fatal: {e}");
            }
        }

        app::ScreenSaverMode::Preview(hwnd) => {
            if let Err(e) = app::run_preview(hwnd, config) {
                eprintln!("screensaver preview: {e}");
            }
        }

        app::ScreenSaverMode::Configure(hwnd) => {
            if let Err(e) = app::run_config(hwnd, config) {
                eprintln!("screensaver config: {e}");
            }
        }
    }
}
