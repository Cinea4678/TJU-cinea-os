



use lazy_static::lazy_static;
use rusttype::{Font, HMetrics, Scale, ScaledGlyph};





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


