use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::config::TimeFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockDigits {
    pub digits: [u8; 4],
    pub is_pm: bool,
}

impl ClockDigits {
    pub fn now(format: TimeFormat) -> Self {
        unsafe {
            let time = GetLocalTime();

            let hour = time.wHour as u8;
            let minute = time.wMinute as u8;

            let display_hour = match format {
                TimeFormat::H12 => match hour % 12 {
                    0 => 12,
                    h => h,
                },
                TimeFormat::H24 => hour,
            };

            Self {
                digits: [
                    display_hour / 10,
                    display_hour % 10,
                    minute / 10,
                    minute % 10,
                ],
                is_pm: hour >= 12,
            }
        }
    }
}
