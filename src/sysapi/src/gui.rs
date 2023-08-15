use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::min;

use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::raw::RawU24;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RawData;
use tinybmp::{Bmp, ChannelMasks, RawBmp, RawPixel};

use crate::call::{CREATE_WINDOW, DESTROY_WINDOW, DISPLAY_FONT_STRING, LOAD_FONT};
use crate::syscall;

pub const WINDOW_WIDTH: usize = 300;
pub const WINDOW_HEIGHT: usize = 200;
pub const WINDOW_CONTENT_WIDTH: usize = 296;
pub const WINDOW_CONTENT_HEIGHT: usize = 178;

pub type WindowGraphicMemory = [[Rgb888; WINDOW_CONTENT_WIDTH]; WINDOW_CONTENT_HEIGHT];

pub struct WindowWriter {
    gm: Box<WindowGraphicMemory>,
    bg_color: Rgb888,
}

impl WindowWriter {
    pub fn new(background_color: Rgb888) -> Self {
        let gm = Box::new([[background_color; WINDOW_CONTENT_WIDTH]; WINDOW_CONTENT_HEIGHT]);
        Self {
            gm,
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

    pub fn display_resolved(&mut self, x: i32, y: i32, resolved: &Vec<(i32, i32, Rgb888)>) {
        for pixel in resolved {
            self.display_pixel_safe((x + pixel.0) as usize, (y + pixel.1) as usize, pixel.2);
        }
    }

    pub fn clear_resolved(&mut self, x: i32, y: i32, resolved: &Vec<(i32, i32, Rgb888)>) {
        for pixel in resolved {
            self.clear_pixel_safe((x + pixel.0) as usize, (y + pixel.1) as usize);
        }
    }

    pub fn resolve_img(bmp_data: &[u8]) -> Option<Vec<(i32, i32, Rgb888)>> {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                let mut res = Vec::new();
                res.reserve(bmp.pixels().count());
                for Pixel(position, color) in bmp.pixels() {
                    res.push((position.y, position.x, color))
                }
                Some(res)
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn resolve_img_32rgba(bmp_data: &[u8]) -> Option<Vec<(i32, i32, Rgb888)>> {
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
                        result.push((position.y, position.x, rgb_color))
                    }
                }
                Some(result)
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn display_font_string(&mut self, s: &str, font_name: &str, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
        let color: RawU24 = color.into();
        let color: u32 = color.into_inner();
        syscall_with_serialize!(DISPLAY_FONT_STRING,(self.gm.as_mut_ptr() as usize,String::from(s),String::from(font_name),x_pos,y_pos,size,line_height,color));
    }

}

pub fn init_window_gui(title: &str, background_color: Rgb888) -> Option<WindowWriter> {
    let writer = WindowWriter::new(background_color);
    let addr = writer.gm.as_ptr() as usize;
    let ret: Result<bool, _> = syscall_with_serdeser!(CREATE_WINDOW,(String::from(title), addr));
    if let Ok(res) = ret && res == true {
        Some(writer)
    } else {
        None
    }
}

pub fn remove_window_gui(_writer: WindowWriter) {
    unsafe { syscall!(DESTROY_WINDOW) };
} 

pub fn load_font(name: &str, path: &str) -> bool {
    let ret: Result<bool, _> = syscall_with_serdeser!(LOAD_FONT, (String::from(name),String::from(path)));
    match ret {
        Ok(ret) => ret,
        Err(_) => false
    }
}


