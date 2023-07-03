use alloc::format;
use core::cmp::min;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics::pixelcolor::Bgr888;
use lazy_static::lazy_static;
use spin::Mutex;
use tinybmp::Bmp;
use volatile::Volatile;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, Page, Size4KiB};
use x86_64::VirtAddr;

use crate::qemu::qemu_print;

pub mod vbe;
pub mod font;

// 相关配置
const WIDTH: usize = 800;
const HEIGHT: usize = 800;

// 相关数据结构

/// 提交到内存的像素
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

/// 屏幕
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<Pixel>; WIDTH]; HEIGHT],
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
    /// color是一个按照_RGB格式给出颜色的数字
    ///
    /// 因为这个函数在关键路径上，所以就不检查边界了
    pub unsafe fn display_pixel(&mut self, x: usize, y: usize, color: u32) {
        self.0.chars[x][y].write(Pixel {
            r: color as u8,
            g: (color >> 8) as u8,
            b: (color >> 16) as u8,
        });
    }

    /// 写像素
    /// color是RGB888
    ///
    /// 因为这个函数在关键路径上，所以就不检查边界了
    pub unsafe fn display_pixel_rgb888(&mut self, x: usize, y: usize, color: Rgb888) {
        self.0.chars[x][y].write(Pixel {
            b: color.r(),
            g: color.g(),
            r: color.b(),
        });
    }

    pub fn display_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
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
                    unsafe { self.display_pixel_rgb888(x + position.y as usize, y + position.x as usize, color); };
                }
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
            }
        }
    }
}



