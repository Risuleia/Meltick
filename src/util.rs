use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO},
        System::Registry::{HKEY, REG_DWORD, REG_VALUE_TYPE, RegQueryValueExW, RegSetValueExW},
    },
    core::BOOL,
};

pub fn parse_hwnd(value: &str) -> Option<HWND> {
    let value = value.trim();

    let raw = if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).ok()?
    } else {
        value.parse::<usize>().ok()?
    };

    if raw == 0 {
        return None;
    }

    Some(HWND(raw as *mut _))
}

pub fn enumerate_monitors() -> windows::core::Result<Vec<RECT>> {
    let mut monitors = Vec::<RECT>::new();

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut Vec<RECT> as isize),
        );
    }

    Ok(monitors)
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let monitors = &mut *(lparam.0 as *mut Vec<RECT>);

        let mut info =
            MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };

        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            monitors.push(info.rcMonitor);
        }
    }

    BOOL(1)
}

pub unsafe fn read_dword(key: HKEY, value_name: windows::core::PCWSTR) -> Option<u32> {
    let mut value_type = REG_VALUE_TYPE::default();
    let mut value = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;

    unsafe {
        let result = RegQueryValueExW(
            key,
            value_name,
            None,
            Some(&mut value_type),
            Some(&mut value as *mut u32 as *mut u8),
            Some(&mut size),
        );

        if result.is_err() {
            return None;
        }
    }

    if value_type != REG_DWORD || size != 4 {
        return None;
    }

    Some(value)
}

pub unsafe fn write_dword(
    key: HKEY,
    value_name: windows::core::PCWSTR,
    value: u32,
) -> windows::core::Result<()> {
    unsafe {
        let result =
            RegSetValueExW(key, value_name, Some(0), REG_DWORD, Some(&value.to_ne_bytes()));

        if result.is_err() {
            return Err(windows::core::Error::from_thread());
        }
    }

    Ok(())
}
