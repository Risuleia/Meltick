use windows::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::HDC,
};

use crate::app::config::{
    constants::COMPONENT_RADIUS,
    draw::{BG, SURFACE, SURFACE_ACTIVE, TEXT, draw_button_text, draw_rounded_rect},
};

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    Secondary,
    Primary,
}

pub struct Button {
    pub rect: RECT,
    pub label: &'static str,
    pub style: ButtonStyle,
}

impl Button {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        label: &'static str,
        style: ButtonStyle,
    ) -> Self {
        Self { rect: RECT { left: x, top: y, right: x + width, bottom: y + height }, label, style }
    }

    pub fn contains(&self, point: POINT) -> bool {
        point.x >= self.rect.left
            && point.x < self.rect.right
            && point.y >= self.rect.top
            && point.y < self.rect.bottom
    }

    pub unsafe fn draw(&self, hdc: HDC, hovered: bool, pressed: bool) {
        let (color, text_color) = match self.style {
            ButtonStyle::Primary => (TEXT, BG),

            ButtonStyle::Secondary => {
                let color = if hovered || pressed { SURFACE_ACTIVE } else { SURFACE };

                (color, TEXT)
            }
        };

        unsafe {
            draw_rounded_rect(hdc, self.rect, COMPONENT_RADIUS, color);

            draw_button_text(hdc, self.label, self.rect, text_color);
        }
    }
}
