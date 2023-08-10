use alloc::boxed::Box;
use alloc::string::String;
use alloc::{format, vec};
use alloc::vec::Vec;
use core::cmp::min;
use core::pin::Pin;
use embedded_graphics::Pixel;

use embedded_graphics::pixelcolor::Rgb888;
use postcard::Error;
use rusttype::{point, Rect, ScaledGlyph};
use tinybmp::{Bmp, ChannelMasks, RawBmp, RawPixel};
use crate::call::CREATE_WINDOW;
use crate::font::get_glyph;
use crate::syscall::log;
use crate::window;

pub const WINDOW_WIDTH: usize = 300;
pub const WINDOW_HEIGHT: usize = 200;
pub const WINDOW_CONTENT_WIDTH: usize = 296;
pub const WINDOW_CONTENT_HEIGHT: usize = 178;

pub type WindowGraphicMemory = Vec<Vec<Rgb888>>;

pub struct WindowWriter {
    gm: Pin<Box<WindowGraphicMemory>>,
    bg_color: Rgb888,
}

impl WindowWriter {
    pub fn new(background_color: Rgb888) -> Self {
        Self {
            gm: Pin::new(Box::new(vec![vec![background_color; WINDOW_CONTENT_WIDTH]; WINDOW_CONTENT_HEIGHT])),
            bg_color: background_color,
        }
    }

    pub unsafe fn display_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        self.gm[x][y] = color;
    }

    pub unsafe fn clear_pixel(&mut self, x: usize, y: usize) {
        self.gm[x][y] = self.bg_color;
    }

    pub fn display_pixel_safe(&mut self, x: usize, y: usize, color: Rgb888) {
        if x < WINDOW_CONTENT_HEIGHT && y < WINDOW_CONTENT_WIDTH {
            self.gm[x][y] = color;
        }
    }

    pub fn clear_pixel_safe(&mut self, x: usize, y: usize) {
        if x < WINDOW_CONTENT_HEIGHT && y < WINDOW_CONTENT_WIDTH {
            self.gm[x][y] = self.bg_color;
        }
    }

    pub fn display_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Rgb888) {
        let x_end = min(x + h, WINDOW_CONTENT_HEIGHT - 1);
        let y_end = min(y + w, WINDOW_CONTENT_WIDTH - 1);
        for i in x..x_end {
            for j in y..y_end {
                unsafe { self.display_pixel(i, j, color); };
            }
        }
    }

    pub fn clear_rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let x_end = min(x + h, WINDOW_CONTENT_HEIGHT - 1);
        let y_end = min(y + w, WINDOW_CONTENT_WIDTH - 1);
        for i in x..x_end {
            for j in y..y_end {
                unsafe { self.clear_pixel(i, j); };
            }
        }
    }

    pub fn display_resolved(&mut self, x: usize, y: usize, resolved: &Vec<(usize, usize, Rgb888)>) {
        for pixel in resolved {
            self.display_pixel_safe(x + pixel.0, y + pixel.1, pixel.2);
        }
    }

    pub fn clear_resolved(&mut self, x: usize, y: usize, resolved: &Vec<(usize, usize, Rgb888)>) {
        for pixel in resolved {
            self.clear_pixel_safe(x + pixel.0, y + pixel.1);
        }
    }

    // FIXME： 有条件后换成i32
    pub fn resolve_img(bmp_data: &[u8]) -> Option<Vec<(usize, usize, Rgb888)>> {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                Some(bmp.pixels().map(|Pixel(position, color)| { (position.y as usize, position.x as usize, color) }).collect())
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn resolve_img_32rgba(bmp_data: &[u8]) -> Option<Vec<(usize, usize, Rgb888)>> {
        let mut result = Vec::new();
        match RawBmp::from_slice(bmp_data) {
            Ok(bmp) => {
                let cm = match bmp.header().channel_masks {
                    None => {
                        ChannelMasks {
                            blue: 0x000000FF,
                            green: 0x0000FF00,
                            red: 0x00FF0000,
                            alpha: 0xFF000000,
                        }
                    },
                    Some(cm) => cm
                };
                let (mut rr, mut br, mut gr, mut ar) = (0, 0, 0, 0);
                let mut rm = cm.red;
                while rm & 1 == 0 {
                    rr += 1;
                    rm >>= 1;
                }
                let mut gm = cm.green;
                while gm & 1 == 0 {
                    gr += 1;
                    gm >>= 1;
                }
                let mut bm = cm.blue;
                while bm & 1 == 0 {
                    br += 1;
                    bm >>= 1;
                }
                let mut am = cm.alpha;
                while am & 1 == 0 {
                    ar += 1;
                    am >>= 1;
                }
                let asize = (cm.alpha >> ar) as f32;

                for RawPixel { position, color } in bmp.pixels() {
                    let rgb_color = Rgb888::new(((color & cm.red) >> rr) as u8, ((color & cm.green) >> gr) as u8, ((color & cm.blue) >> br) as u8);
                    let alpha = ((color & cm.alpha) >> ar) as f32 / asize;
                    if alpha > 0.5 {
                        result.push((position.y as usize, position.x as usize, rgb_color))
                    }
                }
                Some(result)
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn display_font(&mut self, glyph: ScaledGlyph, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
        let bbox = glyph.exact_bounding_box().unwrap_or(Rect {
            min: point(0.0, 0.0),
            max: point(size, size),
        });

        let x_offset = (line_height as f32 + bbox.min.y) as usize;

        let glyph = glyph.positioned(point(0.0, 0.0));
        glyph.draw(|y, x, v| {
            if v > 0.5 {
                self.display_pixel_safe(x_offset + x_pos + x as usize, y_pos + y as usize + bbox.min.x as usize, color);
            }
        });
    }

    /// 敬请注意：此方法不检查换行
    pub unsafe fn display_font_string(&mut self, s: &str, font_name: &str, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
        let mut y_pos = y_pos;
        for ch in s.chars() {
            if y_pos >= WINDOW_CONTENT_WIDTH { return; }
            if let Some((glyph, hm)) = get_glyph(font_name, ch, size) {
                self.display_font(glyph, x_pos, y_pos, size, line_height, color);
                y_pos += hm.advance_width as usize + 1usize;
            }
        }
    }
}

pub fn init_window_gui(title: &str, background_color: Rgb888) -> Option<WindowWriter> {
    let writer = WindowWriter::new(background_color);
    let addr = &*writer.gm as *const Vec<Vec<Rgb888>> as usize;
    let ret: Result<bool, _> = syscall_with_serdeser!(CREATE_WINDOW,(String::from(title), addr));
    if let Ok(res) = ret && res == true {
        Some(writer)
    } else {
        None
    }
}


