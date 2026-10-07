use windows::{
    Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, RegCloseKey,
        RegCreateKeyExW, RegOpenKeyExW,
    },
    core::w,
};

use crate::util::{read_dword, write_dword};

const REGISTRY_PATH: windows::core::PCWSTR = w!("Software\\MeltickClock");

const VALUE_TIME_FORMAT: windows::core::PCWSTR = w!("TimeFormat");

const VALUE_SCALE: windows::core::PCWSTR = w!("Scale");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    H24,
    H12,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub time_format: TimeFormat,
    pub scale: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_format: TimeFormat::H24,
            scale: 1.0,
        }
    }
}

impl Config {
    pub const MIN_SCALE: f32 = 0.50;
    pub const MAX_SCALE: f32 = 1.25;

    pub fn validate(&mut self) {
        self.scale = self.scale.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
    }

    pub fn load() -> Self {
        let mut config = Self::default();

        unsafe {
            let mut key = HKEY::default();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                REGISTRY_PATH,
                Some(0),
                KEY_READ,
                &mut key,
            );

            if result.is_err() {
                return config;
            }

            if let Some(time_format) = read_dword(key, VALUE_TIME_FORMAT) {
                config.time_format = match time_format {
                    12 => TimeFormat::H12,
                    24 => TimeFormat::H24,
                    _ => config.time_format,
                }
            }

            if let Some(scale_bits) = read_dword(key, VALUE_SCALE) {
                config.scale = f32::from_bits(scale_bits);
            }

            let _ = RegCloseKey(key);
        }

        config.validate();

        config
    }

    pub fn save(&self) -> windows::core::Result<()> {
        let mut config = *self;
        config.validate();

        unsafe {
            let mut key = HKEY::default();
            let disposition = 0u32;

            let result = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                REGISTRY_PATH,
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut key,
                Some(disposition as *mut _),
            );

            if result.is_err() {
                return Err(windows::core::Error::from_thread());
            }

            let time_format = match config.time_format {
                TimeFormat::H12 => 12u32,
                TimeFormat::H24 => 24u32,
            };

            write_dword(key, VALUE_TIME_FORMAT, time_format)?;

            write_dword(key, VALUE_SCALE, config.scale.to_bits())?;

            let _ = RegCloseKey(key);
        }

        Ok(())
    }
}
