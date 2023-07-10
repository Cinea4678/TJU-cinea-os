use alloc::format;
use alloc::string::ToString;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use rusttype::{Font, HMetrics, point, PositionedGlyph, Scale, ScaledGlyph};

use crate::graphic::{GD, Writer};
use crate::graphic::text::TextWriter;
use crate::io::qemu::qemu_print;

const FONT_DATA: &[u8] = include_bytes!("../../assets/VonwaonBitmap-16px.ttf");

lazy_static! {
    pub(super) static ref FONT: Font<'static> = Font::try_from_bytes(FONT_DATA).unwrap();
}

pub fn get_font(ch: char, size: f32) -> (ScaledGlyph<'static>, HMetrics){
    let scale = Scale::uniform(size);
    let glyph_id = FONT.glyph(ch);
    let glyph = glyph_id.scaled(scale);
    let h_metrics = glyph.h_metrics();
    (glyph,h_metrics)
}


