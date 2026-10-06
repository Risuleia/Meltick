use windows::Win32::System::SystemInformation::GetLocalTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    TwelveHour,
    #[allow(unused)]
    TwentyFourHour,
}

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
                TimeFormat::TwelveHour => match hour % 12 {
                    0 => 12,
                    h => h,
                },
                TimeFormat::TwentyFourHour => hour,
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
