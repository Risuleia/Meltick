use windows::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::HDC,
};

use crate::{
    app::config::draw::{SURFACE, TEXT, draw_circle, draw_rounded_rect, draw_slider_text},
    config::Config,
};

const SNAP_SCALE: f32 = 1.0;
const SNAP_DISTANCE: f32 = 0.04;

pub struct Slider {
    pub rect: RECT,
}

impl Slider {
    pub const THUMB_RADIUS: i32 = 9;

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { rect: RECT { left: x, top: y, right: x + width, bottom: y + height } }
    }

    pub fn contains(&self, point: POINT) -> bool {
        point.x >= self.rect.left
            && point.x < self.rect.right
            && point.y >= self.rect.top
            && point.y < self.rect.bottom
    }

    pub fn scale_from_x(&self, x: i32) -> f32 {
        let min_x = self.rect.left;
        let max_x = self.rect.right;

        let t = ((x - min_x) as f32 / (max_x - min_x) as f32).clamp(0.0, 1.0);

        let mut scale = Config::MIN_SCALE + t * (Config::MAX_SCALE - Config::MIN_SCALE);

        if (scale - SNAP_SCALE).abs() <= SNAP_DISTANCE {
            scale = SNAP_SCALE;
        }

        scale
    }

    pub fn scale_from_point(&self, x: i32) -> f32 {
        self.scale_from_x(x)
    }

    fn x_from_scale(&self, scale: f32) -> i32 {
        let scale = scale.clamp(Config::MIN_SCALE, Config::MAX_SCALE);

        let t = (scale - Config::MIN_SCALE) / (Config::MAX_SCALE - Config::MIN_SCALE);

        let width = self.rect.right - self.rect.left;

        self.rect.left + (t * width as f32) as i32
    }

    unsafe fn draw_snap_marker(&self, hdc: HDC) {
        let x = self.x_from_scale(1.0);

        let center_y = (self.rect.top + self.rect.bottom) / 2;

        let marker_y = center_y + 13;
        let radius = 3;

        unsafe { draw_circle(hdc, x, marker_y, radius, TEXT) };
    }

    unsafe fn draw_track(&self, hdc: HDC) {
        let center_y = (self.rect.top + self.rect.bottom) / 2;

        let track_height = 6;

        let track = RECT {
            left: self.rect.left,
            top: center_y - track_height / 2,
            right: self.rect.right,
            bottom: center_y + track_height / 2,
        };

        unsafe { draw_rounded_rect(hdc, track, track_height, SURFACE) };
    }

    unsafe fn draw_progress(&self, hdc: HDC, scale: f32) {
        let center_y = (self.rect.top + self.rect.bottom) / 2;

        let x = self.x_from_scale(scale);

        let track_height = 6;

        let progress = RECT {
            left: self.rect.left,
            top: center_y - track_height / 2,
            right: x,
            bottom: center_y + track_height / 2,
        };

        if progress.right > progress.left {
            unsafe { draw_rounded_rect(hdc, progress, track_height / 2, TEXT) };
        }
    }

    unsafe fn draw_thumb(&self, hdc: HDC, scale: f32) {
        let x = self.x_from_scale(scale);

        let center_y = (self.rect.top + self.rect.bottom) / 2;

        unsafe { draw_circle(hdc, x, center_y, Self::THUMB_RADIUS, TEXT) };
    }

    unsafe fn draw_labels(&self, hdc: HDC, scale: f32) {
        let center_y = (self.rect.top + self.rect.bottom) / 2;

        unsafe {
            draw_slider_text(
                hdc,
                "50%",
                RECT {
                    left: self.rect.left - 55,
                    top: center_y - 13,
                    right: self.rect.left - 10,
                    bottom: center_y + 13,
                },
                TEXT,
            );

            draw_slider_text(
                hdc,
                &format!("{:.0}%", scale * 100.0),
                RECT {
                    left: self.x_from_scale(scale) - 35,
                    top: center_y - 42,
                    right: self.x_from_scale(scale) + 35,
                    bottom: center_y - 16,
                },
                TEXT,
            );

            draw_slider_text(
                hdc,
                "125%",
                RECT {
                    left: self.rect.right + 10,
                    top: center_y - 13,
                    right: self.rect.right + 65,
                    bottom: center_y + 13,
                },
                TEXT,
            );
        }
    }

    pub unsafe fn draw(&self, hdc: HDC, scale: f32) {
        unsafe {
            self.draw_track(hdc);
            self.draw_progress(hdc, scale);
            self.draw_snap_marker(hdc);
            self.draw_thumb(hdc, scale);
            self.draw_labels(hdc, scale);
        }
    }
}
