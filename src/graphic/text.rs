use alloc::format;
use core::cmp::min;
use core::fmt;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use rusttype::{Point, point, Rect, Scale, ScaledGlyph};
use spin::Mutex;
use x86_64::instructions::interrupts;

use crate::graphic::{DEFAULT_RGB888, GD, GL, rgb888, Writer};
use crate::graphic::font::get_font;
use crate::io::qemu::qemu_print;

use super::font::FONT;

/// 提交到内存中的HD字符
#[derive(Debug, Clone)]
#[repr(C)]
struct ScreenChar {
    character: ScaledGlyph<'static>,
    color: Rgb888,
}

const TEXT_AREA_HEIGHT: usize = super::HEIGHT - 30;
const TEXT_AREA_WIDTH: usize = super::WIDTH;
const TEXT_AREA_POS: (usize, usize) = (22, 0);
const TEXT_SIZE: f32 = 16.0;
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
    layer: usize,
}

lazy_static! {
    pub static ref TEXT_WRITER: Mutex<TextWriter> = {
        Mutex::new(TextWriter{
            y_position: 0,
            line_position: 0,
            line_height: TEXT_HEIGHT,
            line_gap: 4,
            max_line: TEXT_AREA_HEIGHT / (TEXT_HEIGHT+4),
            color: rgb888!(TEXT_COLOR),
            layer: 1
        })
    };
}

impl TextWriter {
    fn _write_char(&mut self, ch: char) {
        match ch {
            '\t' => self.horizontal_tab(),
            '\n' => self.new_line(),
            ch => {
                let (glyph, hm) = get_font(ch, TEXT_SIZE);
                if self.y_position + hm.advance_width as usize > TEXT_AREA_WIDTH {
                    self.new_line();
                }

                let p_lock = GL.read();
                let mut lock = p_lock[self.layer].lock();
                lock.display_font(glyph, (self.line_height + self.line_gap) * self.line_position + TEXT_AREA_POS.0,
                                  self.y_position + TEXT_AREA_POS.1, TEXT_SIZE, self.line_height, rgb888!(0xddddddu32));

                drop(lock);

                self.y_position += hm.advance_width as usize + 1usize;
            }
        }
    }

    /// 提供外部调用的版本，内部勿调用
    pub fn write_char(&mut self, ch: char) {
        self._write_char(ch);

        GD.lock().render((self.line_height + self.line_gap) * self.line_position + TEXT_AREA_POS.0,
                         self.y_position + TEXT_AREA_POS.1,
                         (self.line_height + self.line_gap) * self.line_position + TEXT_AREA_POS.0 + (TEXT_SIZE * 1.5) as usize,
                         self.y_position + TEXT_AREA_POS.1 + TEXT_SIZE as usize);
    }

    pub fn write_string(&mut self, s: &str) {
        let sx = self.line_position;
        for ch in s.chars() {
            self._write_char(ch);
        }
        let ex = self.line_position;
        GD.lock().render((self.line_height + self.line_gap) * sx + TEXT_AREA_POS.0,
                         TEXT_AREA_POS.1,
                         (self.line_height + self.line_gap) * ex + TEXT_AREA_POS.0 + (TEXT_SIZE * 1.5) as usize,
                         TEXT_AREA_POS.1 + TEXT_AREA_WIDTH);
    }


    fn new_line(&mut self) {
        // 1. 回车
        self.y_position = 0;
        // 2. 加新行
        if self.line_position + 1 < self.max_line {
            self.line_position += 1;
        } else {
            let p_lock = GL.read();
            let mut lock = p_lock[self.layer].lock();
            for x in TEXT_AREA_POS.0..TEXT_AREA_POS.0 + (self.max_line - 1) * (self.line_height + self.line_gap) {
                for y in TEXT_AREA_POS.1..TEXT_AREA_POS.1 + TEXT_AREA_WIDTH {
                    lock.data[x][y] = lock.data[x + (self.line_height + self.line_gap)][y];
                }
            }
            for x in TEXT_AREA_POS.0 + (self.max_line - 1) * (self.line_height + self.line_gap)..TEXT_AREA_POS.0 + TEXT_AREA_HEIGHT {
                for y in TEXT_AREA_POS.1..TEXT_AREA_POS.1 + TEXT_AREA_WIDTH {
                    lock.data[x][y] = (DEFAULT_RGB888, false);
                }
            }

            drop(lock);
            GD.lock().render(TEXT_AREA_POS.0, TEXT_AREA_POS.1, TEXT_AREA_POS.0 + TEXT_AREA_HEIGHT, TEXT_AREA_POS.1 + TEXT_AREA_WIDTH);
        }
    }

    fn backspace(&mut self) {
        unimplemented!()
    }

    fn horizontal_tab(&mut self) {
        unimplemented!()
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.write_string(s);
        Ok(())
    }
}
