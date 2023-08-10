use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use lazy_static::lazy_static;
use rusttype::{Font, HMetrics, Scale, ScaledGlyph};
use spin::RwLock;
use crate::fs;

lazy_static! {
    static ref FONT_MAP: RwLock<BTreeMap<String, Font<'static>>> = {
        RwLock::new(BTreeMap::new())
    };
}

pub fn get_font(name: &str) -> Option<Font<'static>> {
    FONT_MAP.read().get(name).cloned()
}

pub fn load_font(name: &str, path: &str) {
    if FONT_MAP.read().contains_key(name) { return; }
    if let Ok(metadata) = fs::info(path) && metadata.is_file() {
        if let Ok(handle) = fs::open(path, false) {
            let mut buf = vec![0u8; metadata.len() as usize];
            if fs::read(handle, buf.as_mut_slice()).is_ok() {
                if let Some(font) = Font::try_from_vec(buf) {
                    let mut lock = FONT_MAP.write();
                    lock.insert(String::from(name), font);
                }
            }
        }
    }
}

pub fn get_glyph(name: &str, ch: char, size: f32) -> Option<(ScaledGlyph<'static>, HMetrics)> {
    if let Some(font) = get_font(name){
        let scale = Scale::uniform(size);
        let glyph_id = font.glyph(ch);
        let glyph = glyph_id.scaled(scale);
        let h_metrics = glyph.h_metrics();
        Some((glyph,h_metrics))
    } else {
        None
    }
}