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
    Exit, // bad args: do nothing
}

pub fn parse_screen_saver_mode() -> ScreenSaverMode {
    let mut args = std::env::args_os().skip(1);

    let Some(first) = args.next() else {
        return if cfg!(debug_assertions) {
            ScreenSaverMode::Dev
        } else {
            ScreenSaverMode::Configure(None)
        };
    };

    let first = first.to_string_lossy().to_ascii_lowercase();
    if first == "--dev" {
        return ScreenSaverMode::Dev;
    }

    // accept "/s", "-s", "/c:123", "/p 123", "/P:123", ...
    let flag = first.trim_start_matches(['/', '-']);
    let (cmd, inline) = match flag.split_once(':') {
        Some((c, v)) => (c, Some(v.to_string())),
        None => (flag, None),
    };

    // handle comes either after the colon or as the next arg
    let mut handle = || {
        inline
            .clone()
            .or_else(|| args.next().map(|v| v.to_string_lossy().into_owned()))
            .and_then(|v| parse_hwnd(&v))
    };

    match cmd {
        "s" => ScreenSaverMode::ScreenSaver,
        "p" => match handle() {
            Some(hwnd) => ScreenSaverMode::Preview(hwnd),
            None => ScreenSaverMode::Exit,
        },
        "c" => ScreenSaverMode::Configure(handle()),
        _ => ScreenSaverMode::Exit,
    }
}
