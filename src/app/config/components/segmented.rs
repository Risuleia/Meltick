use windows::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::HDC,
};

use crate::{
    app::config::{constants::COMPONENT_RADIUS, draw::{
        SURFACE, SURFACE_ACTIVE, TEXT, draw_centered_text, draw_control_text, draw_rounded_rect,
    }}, config::TimeFormat,
};

pub struct SegmentedControl {
    pub rect: RECT,
}

impl SegmentedControl {
    const INSET: i32 = 4;

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            rect: RECT {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            },
        }
    }

    pub fn contains(&self, point: POINT) -> bool {
        point.x >= self.rect.left
            && point.x < self.rect.right
            && point.y >= self.rect.top
            && point.y < self.rect.bottom
    }

    pub fn value_at(&self, point: POINT) -> Option<TimeFormat> {
        if !self.contains(point) {
            return None;
        }

        let width = self.rect.right - self.rect.left;
        let midpoint = self.rect.left + width / 2;

        if point.x < midpoint {
            Some(TimeFormat::H12)
        } else {
            Some(TimeFormat::H24)
        }
    }

    pub unsafe fn draw(&self, hdc: HDC, selected: TimeFormat) {
        let width = self.rect.right - self.rect.left;
        let height = self.rect.bottom - self.rect.top;
        let half_width = width / 2;

        // Outer pill.
        draw_rounded_rect(hdc, self.rect, COMPONENT_RADIUS, SURFACE);

        // Selected segment.
        let active_rect = match selected {
            TimeFormat::H12 => RECT {
                left: self.rect.left + Self::INSET,
                top: self.rect.top + Self::INSET,
                right: self.rect.left + half_width,
                bottom: self.rect.bottom - Self::INSET,
            },

            TimeFormat::H24 => RECT {
                left: self.rect.left + half_width,
                top: self.rect.top + Self::INSET,
                right: self.rect.right - Self::INSET,
                bottom: self.rect.bottom - Self::INSET,
            },
        };

        draw_rounded_rect(hdc, active_rect, COMPONENT_RADIUS - Self::INSET, SURFACE_ACTIVE);

        draw_control_text(
            hdc,
            "12",
            RECT {
                left: self.rect.left,
                top: self.rect.top,
                right: self.rect.left + half_width,
                bottom: self.rect.bottom,
            },
            TEXT,
        );

        draw_control_text(
            hdc,
            "24",
            RECT {
                left: self.rect.left + half_width,
                top: self.rect.top,
                right: self.rect.right,
                bottom: self.rect.bottom,
            },
            TEXT,
        );
    }
}
