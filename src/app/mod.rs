use windows::Win32::Foundation::HWND;

use crate::util::parse_hwnd;

mod config;
mod dev;
mod preview;
mod screensaver;

pub use config::run_config;
pub use dev::run_dev;
pub use preview::run_preview;
pub use screensaver::run_screensaver;

#[derive(Debug, Clone, Copy)]
pub enum ScreenSaverMode {
    Dev,
    ScreenSaver,
    Preview(HWND),
    Configure(Option<HWND>),
}

pub fn parse_screen_saver_mode() -> ScreenSaverMode {
    let mut args = std::env::args_os();
    let _program = args.next();

    let Some(first) = args.next() else {
        return ScreenSaverMode::Dev;
    };

    let first = first.to_string_lossy().to_ascii_lowercase();

    match first.as_str() {
        "/s" | "-s" => ScreenSaverMode::ScreenSaver,
        "/p" | "-p" => {
            let Some(value) = args.next() else {
                eprintln!("screensaver: /p requires window handle");
                return ScreenSaverMode::Dev;
            };

            match parse_hwnd(&value.to_string_lossy()) {
                Some(hwnd) => ScreenSaverMode::Preview(hwnd),
                None => {
                    eprintln!(
                        "screensaver: invalid preview window handle: {}",
                        value.to_string_lossy()
                    );

                    ScreenSaverMode::Dev
                }
            }
        }

        "/c" | "-c" => {
            let hwnd = args.next().and_then(|value| parse_hwnd(&value.to_string_lossy()));

            ScreenSaverMode::Configure(hwnd)
        }

        _ => ScreenSaverMode::Dev,
    }
}
