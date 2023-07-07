use alloc::format;
use core::arch::asm;
use core::cmp::min;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use lazy_static::lazy_static;
use rusttype::{point, Rect, ScaledGlyph};
use spin::Mutex;
use tinybmp::Bmp;
use volatile::Volatile;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, Page, Size4KiB};
use x86_64::VirtAddr;

use crate::graphic::color::alpha_mix;
use crate::graphic::font::get_font;
use crate::qemu::qemu_print;
use crate::rgb888;

pub mod vbe;
pub mod font;
pub mod text;
pub mod color;

// 相关配置
pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

/// 屏幕
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<Rgb888>; WIDTH]; HEIGHT],
}

/// 显示器
pub struct Writer(&'static mut Buffer);

// 相关常量
lazy_static! {
    pub static ref GD: Mutex<Writer> = {
        Mutex::new(Writer(unsafe {&mut *(Page::<Size4KiB>::containing_address(VirtAddr::new(0xC000_0000)).start_address().as_mut_ptr() as *mut Buffer) }))
    };
}

pub fn enter_wide_mode(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    unsafe { vbe::bga_enter_wide(mapper, frame_allocator); }
}

impl Writer {
    /// 写像素
    /// color是RGB888
    ///
    /// 因为这个函数在关键路径上，所以就不检查边界了
    pub unsafe fn display_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        self.0.chars[x][y].write(color);
    }

    pub fn display_pixel_safe(&mut self, x: usize, y: usize, color: Rgb888) {
        if x < HEIGHT && y < WIDTH {
            self.0.chars[x][y].write(color);
        }
    }

    pub fn display_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Rgb888) {
        let x_end = min(x + h, HEIGHT);
        let y_end = min(y + w, WIDTH);
        qemu_print(format!("{},{},{},{}\n", x, y, x_end, y_end).as_str());
        for i in x..x_end {
            for j in y..y_end {
                unsafe { self.display_pixel(i, j, color); };
            }
        }
    }

    /// 测试函数：画一幅图
    pub fn display_img(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                for Pixel(position, color) in bmp.pixels() {
                    unsafe { self.display_pixel(x + position.y as usize, y + position.x as usize, color); };
                }
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
            }
        }
    }

    pub fn display_font(&mut self, glyph: ScaledGlyph, x_pos: usize, y_pos: usize, size: f32, line_height: usize, fg_color: Rgb888, bg_color: Rgb888) {
        let bbox = glyph.exact_bounding_box().unwrap_or(Rect {
            min: point(0.0, 0.0),
            max: point(size, size),
        });

        let x_offset = (line_height as f32 - (bbox.max.y - bbox.min.y)) as usize;

        let glyph = glyph.positioned(point(0.0, 0.0));
        glyph.draw(|y, x, v| {
            let (color, _) = alpha_mix(fg_color, v, bg_color, 1.0);
            self.display_pixel_safe(x_offset + x_pos + x as usize, y_pos + y as usize + bbox.min.x as usize, color);
        })
    }

    /// 敬请注意：此方法不检查换行
    pub unsafe fn display_font_string(&mut self, s: &str, x_pos: usize, y_pos: usize, size: f32, line_height: usize, fg_color: Rgb888, bg_color: Rgb888) {
        let mut y_pos = y_pos;
        for ch in s.chars() {
            if y_pos >= WIDTH { return; }
            let (glyph, hm) = get_font(ch, size);
            self.display_font(glyph, x_pos, y_pos, size, line_height, fg_color, bg_color);
            y_pos += hm.advance_width as usize + 1usize;
        }
    }
}

pub fn test_img() {
    GD.lock().display_rect(0, 0, 800, 600, rgb888!(0xFFFFFFu32));

    unimplemented!("请为我解除封印");
    // let lpld = include_bytes!("../assets/91527085_p0.bmp");
    // let cinea_os = include_bytes!("../assets/cinea-os.bmp");
    // GD.lock().display_img(0, 0, lpld);
    // GD.lock().display_img(400, 300, cinea_os);
}



