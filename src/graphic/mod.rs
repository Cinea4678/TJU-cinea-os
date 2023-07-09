use alloc::{format, vec};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::min;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use lazy_static::lazy_static;
use pc_keyboard::KeyCode::W;
use rusttype::{point, Rect, ScaledGlyph};
use spin::Mutex;
use tinybmp::{Bmp, RawBmp, RawPixel};
use volatile::Volatile;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, Page, Size4KiB};
use x86_64::VirtAddr;

use crate::graphic::color::{alpha_mix, alpha_mix_final};
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
pub struct PhysicalWriter(&'static mut Buffer);

pub struct Writer {
    data: [[(Rgb888, f32); WIDTH]; HEIGHT],
    //mask: [[u8; WIDTH]; HEIGHT],
}

lazy_static! {
    // 这个是最底层的显存
    pub static ref GD: Mutex<PhysicalWriter> = {
        Mutex::new(PhysicalWriter(unsafe {&mut *(Page::<Size4KiB>::containing_address(VirtAddr::new(0xC000_0000)).start_address().as_mut_ptr() as *mut Buffer) }))
    };

    // 多层叠加显示
    pub static ref GL: Mutex<Vec<Writer>> = {
        Mutex::new(Vec::new())
    };
}

pub fn enter_wide_mode(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    unsafe { vbe::bga_enter_wide(mapper, frame_allocator); }
}

impl PhysicalWriter {
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

        let x_offset = (line_height as f32 + bbox.min.y) as usize;
        //qemu_print(format!("{:?},{:?},{:?}\n",ch,bbox,x_offset).as_str());

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

impl Writer {
    pub fn new() -> Self {
        Self {
            data: [[(rgb888!(0), 0.0); WIDTH]; HEIGHT],
        }
    }

    /// 写像素
    /// color是RGB888
    ///
    /// 因为这个函数在关键路径上，所以就不检查边界了
    pub unsafe fn display_pixel(&mut self, x: usize, y: usize, color: Rgb888, alpha: f32) {
        self.data[x][y] = (color, alpha);
    }

    pub fn display_pixel_safe(&mut self, x: usize, y: usize, color: Rgb888, alpha: f32) {
        if x < HEIGHT && y < WIDTH {
            self.data[x][y] = (color, alpha);
        }
    }

    pub fn display_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Rgb888, alpha: f32) {
        let x_end = min(x + h, HEIGHT);
        let y_end = min(y + w, WIDTH);
        for i in x..x_end {
            for j in y..y_end {
                self.data[x][y] = (color, alpha);
            }
        }
    }

    pub fn display_img(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                for Pixel(position, color) in bmp.pixels() {
                    self.data[x + position.y as usize][y + position.x as usize] = (color, 1.0);
                }
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
            }
        }
    }

    pub fn display_img_32rgba(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        match RawBmp::from_slice(bmp_data) {
            Ok(bmp) => {
                match bmp.header().channel_masks {
                    None => {},
                    Some(cm) => {
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
                            self.display_pixel_safe(position.y as usize, position.x as usize, rgb_color, alpha);
                        }
                    }
                }
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
            }
        }
    }

    pub fn display_font(&mut self, glyph: ScaledGlyph, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
        let bbox = glyph.exact_bounding_box().unwrap_or(Rect {
            min: point(0.0, 0.0),
            max: point(size, size),
        });

        let x_offset = (line_height as f32 + bbox.min.y) as usize;
        //qemu_print(format!("{:?},{:?},{:?}\n",ch,bbox,x_offset).as_str());

        let glyph = glyph.positioned(point(0.0, 0.0));
        glyph.draw(|y, x, v| {
            self.display_pixel_safe(x_offset + x_pos + x as usize, y_pos + y as usize + bbox.min.x as usize, color, v);
        })
    }

    /// 敬请注意：此方法不检查换行
    pub unsafe fn display_font_string(&mut self, s: &str, x_pos: usize, y_pos: usize, size: f32, line_height: usize, color: Rgb888) {
        let mut y_pos = y_pos;
        for ch in s.chars() {
            if y_pos >= WIDTH { return; }
            let (glyph, hm) = get_font(ch, size);
            self.display_font(glyph, x_pos, y_pos, size, line_height, color);
            y_pos += hm.advance_width as usize + 1usize;
        }
    }

    ///将图像整体移动
    pub fn move_to(&mut self, dx: i32, dy: i32) {
        let x_iter: Box<dyn Iterator<Item=usize>> = if dx > 0 {
            Box::new(0..HEIGHT)
        } else {
            Box::new((0..HEIGHT).rev())
        };
        let y_iter: Box<dyn Iterator<Item=usize>> = if dy > 0 {
            Box::new(0..WIDTH)
        } else {
            Box::new((0..WIDTH).rev())
        };
        for i in x_iter {
            for j in y_iter {
                if ((i as i32 - dx) as usize) < HEIGHT && ((j as i32 - dy) as usize) < WIDTH {
                    self.data[i][j] = self.data[(i as i32 - dx) as usize][(j as i32 - dy) as usize];
                } else {
                    self.data[i][j] = (rgb888!(0), 0.0);
                }
            }
        }
    }
}

impl PhysicalWriter {
    pub fn render(&mut self, sx: usize, sy: usize, ex: usize, ey: usize) {
        let lock = GL.lock();
        if lock.len() == 0 { return; }
        let mut graph = lock.last().unwrap().data;
        if sx < HEIGHT && sy < WIDTH && ex <= HEIGHT && ey <= WIDTH {
            for layer in (1..lock.len() - 1).rev() {
                let tomix = lock[layer].data;
                for x in sx..ex {
                    for y in sy..ey {
                        if graph[x][y].1 > 0.99 { continue; }
                        graph[x][y] = alpha_mix(graph[x][y].0, graph[x][y].1, tomix[x][y].0, tomix[x][y].1);
                    }
                }
            }
            let tomix = lock[0].data;
            for x in sx..ex {
                for y in sy..ey {
                    self.0.chars[x][y].write(
                        if graph[x][y].1 > 0.99 { graph[x][y].0 } else { alpha_mix_final(graph[x][y].0, graph[x][y].1, tomix[x][y].0) }
                    );
                }
            }
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



