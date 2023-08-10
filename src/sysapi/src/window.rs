use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;
use core::pin::Pin;

use embedded_graphics::pixelcolor::Rgb888;

pub const WINDOW_WIDTH: usize = 200;
pub const WINDOW_HEIGHT: usize = 200;
pub const WINDOW_CONTENT_WIDTH: usize = 196;
pub const WINDOW_CONTENT_HEIGHT: usize = 180;

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
        if x < HEIGHT && y < WIDTH {
            self.gm[x][y] = color;
        }
    }

    pub fn clear_pixel_safe(&mut self, x: usize, y: usize) {
        if x < HEIGHT && y < WIDTH {
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
}


