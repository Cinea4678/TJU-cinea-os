use alloc::collections::BTreeMap;
use alloc::string::String;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use rusttype::{point, Font, HMetrics, Rect, Scale, ScaledGlyph};
use spin::RwLock;

use cinea_os_sysapi::fs::{read_all_from_path, FileError};
use cinea_os_sysapi::gui::{WindowGraphicMemory, WINDOW_CONTENT_HEIGHT, WINDOW_CONTENT_WIDTH};

lazy_static! {
    static ref FONT_MAP: RwLock<BTreeMap<String, Font<'static>>> = { RwLock::new(BTreeMap::new()) };
}

pub fn get_font(name: &str) -> Option<Font<'static>> {
    FONT_MAP.read().get(name).cloned()
}

pub fn load_font(name: &str, path: &str) -> Result<(), FileError> {
    if FONT_MAP.read().contains_key(name) {
        return Ok(());
    }

    let buf = read_all_from_path(path)?;

    if let Some(font) = Font::try_from_vec(buf) {
        let mut lock = FONT_MAP.write();
        lock.insert(String::from(name), font);
        Ok(())
    } else {
        Err(FileError::DeviceIOError)
    }
}

pub fn get_glyph(name: &str, ch: char, size: f32) -> Option<(ScaledGlyph<'static>, HMetrics)> {
    if let Some(font) = get_font(name) {
        let scale = Scale::uniform(size);
        let glyph_id = font.glyph(ch);
        let glyph = glyph_id.scaled(scale);
        let h_metrics = glyph.h_metrics();
        Some((glyph, h_metrics))
    } else {
        None
    }
}

pub fn display_font(window: &mut WindowGraphicMemory, glyph: ScaledGlyph, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
    let bb = glyph.exact_bounding_box().unwrap_or(Rect {
        min: point(0.0, 0.0),
        max: point(size, size),
    });

    let x_offset = (line_height as f32 + bb.min.y) as usize;

    let glyph = glyph.positioned(point(0.0, 0.0));
    glyph.draw(|y, x, v| {
        if v > 0.5 {
            let dx = x_offset + x_pos + x as usize;
            let dy = y_pos + y as usize + bb.min.x as usize;
            if dx < WINDOW_CONTENT_HEIGHT && dy < WINDOW_CONTENT_WIDTH {
                window[dx][dy] = color;
            }
        }
    });
}

pub fn display_font_string(
    window: &mut WindowGraphicMemory,
    s: &str,
    font_name: &str,
    x_pos: usize,
    y_pos: usize,
    size: f32,
    line_height: usize,
    color: Rgb888,
) {
    let mut y_pos = y_pos;
    for ch in s.chars() {
        if y_pos >= WINDOW_CONTENT_WIDTH {
            return;
        }
        if let Some((glyph, hm)) = get_glyph(font_name, ch, size) {
            display_font(window, glyph, x_pos, y_pos, size, line_height, color);
            y_pos += hm.advance_width as usize + 1usize;
        }
    }
}
