use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::{format, vec};
use lazy_static::lazy_static;
use rusttype::{Font, HMetrics, Scale, ScaledGlyph};
use spin::RwLock;
use crate::fs;
use crate::fs::{FileError, read_all_from_path};
use crate::syscall::log;

lazy_static! {
    static ref FONT_MAP: RwLock<BTreeMap<String, Font<'static>>> = {
        RwLock::new(BTreeMap::new())
    };
}

pub fn get_font(name: &str) -> Option<Font<'static>> {
    FONT_MAP.read().get(name).cloned()
}

pub fn load_font(name: &str, path: &str) -> Result<(), FileError> {
    if FONT_MAP.read().contains_key(name) { return Ok(()); }
    let buf = read_all_from_path(String::from(path))?;
    // log(format!("{:?}",&buf.as_slice()[buf.len()-10..buf.len()]).as_bytes());
    if let Some(font) = Font::try_from_vec(buf) {
        let mut lock = FONT_MAP.write();
        lock.insert(String::from(name), font);
        Ok(())
    } else {
        Err(FileError::DeviceIOError)
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