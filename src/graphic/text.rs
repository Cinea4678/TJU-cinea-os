use alloc::format;
use core::cmp::min;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use rusttype::{Point, point, Rect, Scale, ScaledGlyph};
use spin::Mutex;

use crate::graphic::{GD, rgb888, Writer};
use crate::graphic::font::get_font;
use crate::qemu::qemu_print;

use super::font::FONT;

/// 提交到内存中的HD字符
#[derive(Debug, Clone)]
#[repr(C)]
struct ScreenChar {
    character: ScaledGlyph<'static>,
    color: Rgb888,
}

const TEXT_AREA_HEIGHT: usize = super::HEIGHT - 20;
const TEXT_AREA_WIDTH: usize = super::WIDTH;
const TEXT_AREA_POS: (usize, usize) = (20, 0);
const TEXT_SIZE: f32 = 32.0;
const TEXT_HEIGHT: usize = TEXT_SIZE as usize;
const TEXT_COLOR: usize = 0x000000;
const TAB_SIZE: usize = 4 * 16;

/// 输出器
pub struct TextWriter {
    y_position: usize,
    line_position: usize,
    line_height: usize,
    line_gap: usize,
    max_line: usize,
    color: Rgb888,
    gd: &'static GD,
}

lazy_static! {
    pub static ref TEXT_WRITER: Mutex<TextWriter> = {
        let vm = FONT.v_metrics(Scale::uniform(TEXT_HEIGHT as f32));
        Mutex::new(TextWriter{
            y_position: 0,
            line_position: 0,
            line_height: TEXT_HEIGHT,
            line_gap: 4,
            max_line: TEXT_AREA_HEIGHT / (vm.ascent+vm.line_gap) as usize,
            color: rgb888!(TEXT_COLOR),
            gd: &GD
        })
    };
}

impl TextWriter {
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\t' => self.horizontal_tab(),
            '\n' => self.new_line(),
            ch => {
                let (glyph, hm) = get_font(ch, TEXT_SIZE);
                let bbox = glyph.exact_bounding_box().unwrap_or(Rect {
                    min: point(0.0, 0.0),
                    max: point(TEXT_SIZE, TEXT_SIZE),
                });
                if self.y_position + hm.advance_width as usize > TEXT_AREA_WIDTH {
                    self.new_line();
                }

                let x_offset = (self.line_height as f32 - (bbox.max.y - bbox.min.y));
                qemu_print(format!("{:?},{:?},{:?},{:?}\n", ch, bbox, x_offset,self.line_height).as_str());
                let x_offset = x_offset as usize;

                let glyph = glyph.positioned(point(0.0, 0.0));

                let mut lock = self.gd.lock();

                glyph.draw(|y, x, v| unsafe {
                    let c = (255.0 * v) as u8;
                    let c = Rgb888::new(c, c, c);
                    lock.display_pixel((self.line_height + self.line_gap) * self.line_position + x_offset + x as usize, self.y_position + y as usize, c);
                });

                self.y_position += bbox.max.x as usize + 1usize;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    fn clear_row(&mut self, row: usize) {
        unimplemented!()
    }

    fn new_line(&mut self) {
        unimplemented!()
    }

    fn backspace(&mut self) {
        unimplemented!()
    }

    fn carriage_return(&mut self) {
        unimplemented!()
    }

    fn horizontal_tab(&mut self) {
        unimplemented!()
    }
}