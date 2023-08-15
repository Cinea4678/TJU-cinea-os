use core::fmt;

use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use rusttype::ScaledGlyph;
use spin::Mutex;

use crate::syskrnl::graphic::font::get_font;
use crate::syskrnl::graphic::{rgb888, DEFAULT_RGB888, GL};

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
const TEXT_COLOR: Rgb888 = rgb888!(0xddddddu32);
const TAB_SIZE: usize = 4 * 16;

/// 输出器
pub struct TextWriter {
    pub(crate) text_area_height: usize,
    pub(crate) text_area_width: usize,
    pub(crate) text_area_pos: (usize, usize),
    pub(crate) y_position: usize,
    pub(crate) line_position: usize,
    pub(crate) line_height: usize,
    pub(crate) line_gap: usize,
    pub(crate) max_line: usize,
    pub(crate) color: Rgb888,
    pub(crate) layer: usize,
}

lazy_static! {
    pub static ref TEXT_WRITER: Mutex<TextWriter> = {
        Mutex::new(TextWriter {
            text_area_height: TEXT_AREA_HEIGHT,
            text_area_width: TEXT_AREA_WIDTH,
            text_area_pos: TEXT_AREA_POS,
            y_position: 0,
            line_position: 0,
            line_height: TEXT_HEIGHT,
            line_gap: 4,
            max_line: TEXT_AREA_HEIGHT / (TEXT_HEIGHT + 4),
            color: TEXT_COLOR,
            layer: 1,
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
                if self.y_position + hm.advance_width as usize > self.text_area_width {
                    self.new_line();
                }

                let p_lock = GL.read();
                let mut lock = p_lock[self.layer].lock();
                lock.display_font(
                    glyph,
                    (self.line_height + self.line_gap) * self.line_position + self.text_area_pos.0,
                    self.y_position + self.text_area_pos.1,
                    TEXT_SIZE,
                    self.line_height,
                    self.color,
                );

                drop(lock);

                self.y_position += hm.advance_width as usize + 1usize;
            }
        }
    }

    /// 提供外部调用的版本，内部勿调用
    pub fn write_char(&mut self, ch: char) {
        self._write_char(ch);
    }

    pub fn write_string(&mut self, s: &str) {
        let _sx = self.line_position;
        for ch in s.chars() {
            self._write_char(ch);
        }
        let _ex = self.line_position;
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
            for x in self.text_area_pos.0..self.text_area_pos.0 + (self.max_line - 1) * (self.line_height + self.line_gap) {
                for y in self.text_area_pos.1..self.text_area_pos.1 + self.text_area_width {
                    lock.data[x][y] = lock.data[x + (self.line_height + self.line_gap)][y];
                }
            }
            for x in self.text_area_pos.0 + (self.max_line - 1) * (self.line_height + self.line_gap)..self.text_area_pos.0 + self.text_area_height {
                for y in self.text_area_pos.1..self.text_area_pos.1 + self.text_area_width {
                    lock.data[x][y] = (DEFAULT_RGB888, false);
                }
            }

            drop(lock);
        }
    }

    fn horizontal_tab(&mut self) {
        self.y_position += TAB_SIZE - self.y_position % TAB_SIZE;
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.write_string(s);
        Ok(())
    }
}
