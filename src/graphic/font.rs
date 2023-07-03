use alloc::format;
use alloc::string::ToString;

use lazy_static::lazy_static;
use rusttype::{Font, point, Scale};

use crate::graphic::GD;
use crate::qemu::qemu_print;

const FONT_DATA: &[u8] = include_bytes!("../../assets/VonwaonBitmap-16px.ttf");

lazy_static! {
    static ref FONT: Font<'static> = Font::try_from_bytes(FONT_DATA).unwrap();
}

pub fn test_font() {
    let sample = "Test只因你太美";
    let mut fx = 0.0f32;
    for (i, ch) in sample.chars().enumerate() {
        let scale = Scale::uniform(16.0);
        let glyph_id = FONT.glyph(ch);
        let glyph = glyph_id.scaled(scale);
        let h_metrics = glyph.h_metrics();
        qemu_print(format!("{:?},{:?}", ch, h_metrics).as_str());
        let offset_x = i as f32 * h_metrics.advance_width;
        let offset_y = FONT.v_metrics(scale).ascent;
        let position = point(offset_x, offset_y);
        let glyph = glyph.positioned(position);

        let mut gd = GD.lock();
        glyph.draw(|x, y, v| {
            let c = (255.0 * (1.0 - v)) as u32;
            let c = (c << 16) + (c << 8) + c;
            //qemu_print(format!("{},{},{},#{:X}\n", x, y, v, c).as_str());
            unsafe { gd.display_pixel(y as usize, fx as usize + x as usize, c); };
        });
        fx += h_metrics.advance_width;
    }
}