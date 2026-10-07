use windows::{
    Win32::{
        Foundation::{COLORREF, RECT},
        Graphics::Gdi::{
            CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DT_CENTER,
            DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, FillRect, HALFTONE, HDC,
            RoundRect, SRCCOPY, SelectObject, SetBkMode, SetStretchBltMode, SetTextColor,
            StretchBlt, TRANSPARENT,
        },
    },
    core::w,
};

use crate::{
    app::config::{
        components::{Button, SegmentedControl, Slider},
        window::ButtonId,
    },
    config::Config,
};

pub const BG: COLORREF = COLORREF(0x131313);
pub const TEXT: COLORREF = COLORREF(0xD9D9D9);

pub const SURFACE: COLORREF = COLORREF(0x333333);
pub const SURFACE_ACTIVE: COLORREF = COLORREF(0x232323);

const AA_SCALE: i32 = 4;

pub unsafe fn draw_background(hdc: HDC, rect: &RECT) {
    unsafe {
        let brush = CreateSolidBrush(BG);

        FillRect(hdc, rect, brush);

        let _ = DeleteObject(brush.into());
    }
}

pub unsafe fn draw_rounded_rect(hdc: HDC, rect: RECT, radius: i32, color: COLORREF) {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    if width <= 0 || height <= 0 {
        return;
    }

    let scale = AA_SCALE;

    let aa_width = width * scale;
    let aa_height = height * scale;

    unsafe {
        let mem_dc = CreateCompatibleDC(Some(hdc));

        if mem_dc.is_invalid() {
            return;
        }

        let bitmap = CreateCompatibleBitmap(hdc, aa_width, aa_height);

        if bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            return;
        }

        let old_bitmap = SelectObject(mem_dc, bitmap.into());

        SetStretchBltMode(mem_dc, HALFTONE);

        let _ = StretchBlt(
            mem_dc,
            0,
            0,
            aa_width,
            aa_height,
            Some(hdc),
            rect.left,
            rect.top,
            width,
            height,
            SRCCOPY,
        );

        let brush = CreateSolidBrush(color);
        let old_brush = SelectObject(mem_dc, brush.into());

        let scaled_radius = radius * scale;

        let _ = RoundRect(mem_dc, 0, 0, aa_width, aa_height, scaled_radius * 2, scaled_radius * 2);

        SelectObject(mem_dc, old_brush);

        let _ = DeleteObject(brush.into());

        SetStretchBltMode(hdc, HALFTONE);

        let _ = StretchBlt(
            hdc,
            rect.left,
            rect.top,
            width,
            height,
            Some(mem_dc),
            0,
            0,
            aa_width,
            aa_height,
            SRCCOPY,
        );

        SelectObject(mem_dc, old_bitmap);

        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(mem_dc);
    }
}

pub unsafe fn draw_circle(hdc: HDC, center_x: i32, center_y: i32, radius: i32, color: COLORREF) {
    let size = radius * 2;

    let rect = RECT {
        left: center_x - radius,
        top: center_y - radius,
        right: center_x + radius,
        bottom: center_y + radius,
    };

    let scale = AA_SCALE;

    let aa_size = size * scale;

    unsafe {
        let mem_dc = CreateCompatibleDC(Some(hdc));

        if mem_dc.is_invalid() {
            return;
        }

        let bitmap = CreateCompatibleBitmap(hdc, aa_size, aa_size);

        if bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            return;
        }

        let old_bitmap = SelectObject(mem_dc, bitmap.into());

        SetStretchBltMode(mem_dc, HALFTONE);

        let _ = StretchBlt(
            mem_dc,
            0,
            0,
            aa_size,
            aa_size,
            Some(hdc),
            rect.left,
            rect.top,
            size,
            size,
            SRCCOPY,
        );

        let brush = CreateSolidBrush(color);
        let old_brush = SelectObject(mem_dc, brush.into());

        let null_pen =
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_PEN);

        let old_pen = SelectObject(mem_dc, null_pen);

        let _ = windows::Win32::Graphics::Gdi::Ellipse(mem_dc, 0, 0, aa_size, aa_size);

        SelectObject(mem_dc, old_pen);
        SelectObject(mem_dc, old_brush);

        let _ = DeleteObject(brush.into());

        SetStretchBltMode(hdc, HALFTONE);

        let _ = StretchBlt(
            hdc,
            rect.left,
            rect.top,
            size,
            size,
            Some(mem_dc),
            0,
            0,
            aa_size,
            aa_size,
            SRCCOPY,
        );

        SelectObject(mem_dc, old_bitmap);

        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(mem_dc);
    }
}

pub unsafe fn draw_button_text(hdc: HDC, text: &str, rect: RECT, color: COLORREF) {
    unsafe {
        let mut wide: Vec<u16> = text.encode_utf16().collect();

        let font = CreateFontW(
            -18,
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            windows::Win32::Graphics::Gdi::FONT_CHARSET(0),
            windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
            0,
            windows::core::w!("Segoe UI"),
        );

        let old_font = SelectObject(hdc, font.into());

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color);

        let mut rect = rect;

        DrawTextW(hdc, &mut wide, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);

        let _ = DeleteObject(font.into());
    }
}

pub unsafe fn draw_control_text(hdc: HDC, text: &str, rect: RECT, color: COLORREF) {
    let mut wide: Vec<u16> = text.encode_utf16().collect();

    unsafe {
        let font = CreateFontW(
            -22,
            0,
            0,
            0,
            700,
            0,
            0,
            0,
            windows::Win32::Graphics::Gdi::FONT_CHARSET(0),
            windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
            0,
            w!("Segoe UI"),
        );

        let old_font = SelectObject(hdc, font.into());

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color);

        let mut rect = rect;

        DrawTextW(hdc, &mut wide, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);
        let _ = DeleteObject(font.into());
    }
}

pub unsafe fn draw_slider_text(hdc: HDC, text: &str, rect: RECT, color: COLORREF) {
    let mut wide: Vec<u16> = text.encode_utf16().collect();

    unsafe {
        let font = CreateFontW(
            -19,
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            windows::Win32::Graphics::Gdi::FONT_CHARSET(0),
            windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
            windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
            0,
            w!("Segoe UI"),
        );

        let old_font = SelectObject(hdc, font.into());

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color);

        let mut rect = rect;

        DrawTextW(hdc, &mut wide, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);
        let _ = DeleteObject(font.into());
    }
}

pub unsafe fn draw_config_ui(
    hdc: HDC,
    client: RECT,
    config: &Config,
    format_control: &SegmentedControl,
    scale_control: &Slider,
    defaults_button: &Button,
    cancel_button: &Button,
    apply_button: &Button,
    hovered_button: Option<ButtonId>,
    pressed_button: Option<ButtonId>,
) {
    unsafe {
        draw_background(hdc, &client);

        format_control.draw(hdc, config.time_format);

        scale_control.draw(hdc, config.scale);

        defaults_button.draw(
            hdc,
            hovered_button == Some(ButtonId::Defaults),
            pressed_button == Some(ButtonId::Defaults),
        );

        cancel_button.draw(
            hdc,
            hovered_button == Some(ButtonId::Cancel),
            pressed_button == Some(ButtonId::Cancel),
        );

        apply_button.draw(
            hdc,
            hovered_button == Some(ButtonId::Apply),
            pressed_button == Some(ButtonId::Apply),
        );
    }
}
