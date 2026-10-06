mod app;
mod clock;
mod config;
mod gfx;
mod spring;

fn main() {
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
        );
    }

    if let Err(e) = app::run_window() {
        eprintln!("fatal: {e}");
    }
}
