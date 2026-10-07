use windows::Win32::Foundation::COLORREF;

pub const CONFIG_WIDTH: i32 = 1100;
pub const CONFIG_HEIGHT: i32 = 800;

pub const WINDOW_RADIUS: i32 = 22;
pub const COMPONENT_RADIUS: i32 = 16;

pub const PREVIEW_MARGIN_X: i32 = 85;
pub const PREVIEW_TOP: i32 = 40;
pub const PREVIEW_HEIGHT: i32 = 560;

pub const FORMAT_X: i32 = 85;
pub const FORMAT_Y: i32 = 645;
pub const FORMAT_W: i32 = 102;
pub const FORMAT_H: i32 = 52;

pub const SCALE_X: i32 = 260;
pub const SCALE_Y: i32 = 645;
pub const SCALE_W: i32 = 330;
pub const SCALE_H: i32 = 52;

pub const DEFAULTS_X: i32 = 898;
pub const DEFAULTS_Y: i32 = 645;
pub const DEFAULTS_W: i32 = 120;
pub const DEFAULTS_H: i32 = 52;

pub const CANCEL_X: i32 = 760;
pub const APPLY_X: i32 = 898;
pub const ACTION_Y: i32 = 710;
pub const ACTION_W: i32 = 120;
pub const ACTION_H: i32 = 52;

pub const BG: COLORREF = COLORREF(0x00131313);
pub const CONTROL_BG: COLORREF = COLORREF(0x002F2F2F);
pub const CONTROL_ACTIVE: COLORREF = COLORREF(0x00414141);
pub const TEXT: COLORREF = COLORREF(0x00FFFFFF);
pub const MUTED: COLORREF = COLORREF(0x00D0D0D0);
pub const TRACK: COLORREF = COLORREF(0x00383838);