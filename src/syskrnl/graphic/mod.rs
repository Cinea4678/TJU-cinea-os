use alloc::{format, vec};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::min;
use core::fmt;


use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use lazy_static::lazy_static;
use rusttype::{point, Rect, ScaledGlyph};
use spin::{Mutex, RwLock};
use tinybmp::{Bmp, ChannelMasks, RawBmp, RawPixel};
use volatile::Volatile;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, Page, Size4KiB};
use x86_64::VirtAddr;
use cinea_os_sysapi::gui::WindowGraphicMemory;

use crate::{rgb888};
use crate::syskrnl::graphic::color::alpha_mix;
use crate::syskrnl::graphic::font::get_font;
use crate::syskrnl::graphic::text::TEXT_WRITER;
use crate::syskrnl::io::qemu::qemu_print;
use crate::syskrnl::io::VIDEO_MODE;

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

#[derive(Clone, Debug)]
pub struct Writer {
    pub data: Vec<Vec<(Rgb888, bool)>>,
    pub enable: bool,
}

lazy_static! {
    // 这个是最底层的显存
    pub static ref GD: Mutex<PhysicalWriter> = {
        Mutex::new(PhysicalWriter(unsafe {&mut *(Page::<Size4KiB>::containing_address(VirtAddr::new(0xC000_0000)).start_address().as_mut_ptr() as *mut Buffer) }))
    };

    // 多层叠加显示
    pub static ref GL: RwLock<Vec<Mutex<Writer>>> = {
        let mut v:Vec<Mutex<Writer>> = vec![];
        v.reserve(5);
        for _ in 0..5{
            v.push(Mutex::new(Writer::new()));
        }
        RwLock::new(v)
    };
}

pub fn enter_wide_mode(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    unsafe { vbe::bga_enter_wide(mapper, frame_allocator); }
    VIDEO_MODE.lock().set_graphic();
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

const DEFAULT_RGB888: Rgb888 = Rgb888::new(0, 0, 0);

impl Writer {
    pub fn new() -> Self {
        Self {
            data: vec![vec![(DEFAULT_RGB888, false); WIDTH]; HEIGHT],
            enable: false,
        }
    }

    /// 写像素
    /// color是RGB888
    ///
    /// 因为这个函数在关键路径上，所以就不检查边界了
    pub unsafe fn display_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        self.data[x][y] = (color, true);
    }

    pub unsafe fn clear_pixel(&mut self, x: usize, y: usize) {
        self.data[x][y] = (DEFAULT_RGB888, false);
    }

    pub fn display_pixel_safe(&mut self, x: usize, y: usize, color: Rgb888) {
        if x < HEIGHT && y < WIDTH {
            self.data[x][y] = (color, true);
        }
    }

    pub fn clear_pixel_safe(&mut self, x: usize, y: usize) {
        if x < HEIGHT && y < WIDTH {
            self.data[x][y] = (DEFAULT_RGB888, false);
        }
    }

    pub fn display_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Rgb888) {
        let x_end = min(x + h, HEIGHT);
        let y_end = min(y + w, WIDTH);
        // debugln!("{:?}", self.data[x][y]);
        for i in x..x_end {
            for j in y..y_end {
                self.data[i][j] = (color, true);
            }
        }
        // debugln!("{:?}", self.data[x][y]);
    }

    pub fn clear_rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let x_end = min(x + h, HEIGHT);
        let y_end = min(y + w, WIDTH);
        for i in x..x_end {
            for j in y..y_end {
                self.data[i][j] = (DEFAULT_RGB888, false);
            }
        }
    }

    pub fn display_img(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                for Pixel(position, color) in bmp.pixels() {
                    self.data[x + position.y as usize][y + position.x as usize] = (color, true);
                }
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
            }
        }
    }

    pub fn clear_img(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                let size = bmp.size();
                self.clear_rect(x, y, size.width as usize, size.height as usize);
            }
            Err(error) => {
                qemu_print(format!("{:?}\n", error).as_str());
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

    pub fn display_img_32rgba(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
        let resolved = resolve_32rgba(bmp_data);
        self.display_resolved(x, y, &resolved);
    }

    pub fn clear_img_32rgba(&mut self, x: usize, y: usize, bmp_data: &[u8]) {
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
                let mut ar = 0;
                let mut am = cm.alpha;
                while am & 1 == 0 {
                    ar += 1;
                    am >>= 1;
                }
                let asize = (cm.alpha >> ar) as f32;

                for RawPixel { position, color } in bmp.pixels() {
                    let alpha = ((color & cm.alpha) >> ar) as f32 / asize;
                    //qemu_print(format!("{:?},{:?}", rgb_color, alpha).as_str());
                    if alpha > 0.5 {
                        self.clear_pixel_safe(x + position.y as usize, y + position.x as usize);
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
            if v > 0.5 {
                self.display_pixel_safe(x_offset + x_pos + x as usize, y_pos + y as usize + bbox.min.x as usize, color);
            }
        });
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

    pub fn display_from_copied(&mut self, x: usize, y: usize, data: &WindowGraphicMemory) {
        for (i, c) in data.iter().enumerate() {
            for (j, d) in c.iter().enumerate() {
                self.display_pixel_safe(x + i, y + j, *d);
            }
        }
    }

    ///将图像整体移动
    pub fn move_to(&mut self, dx: i32, dy: i32) {
        let mut new_data = vec![vec![(DEFAULT_RGB888, false); WIDTH]; HEIGHT];

        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                let old_x = (j as i32 - dx) as isize;
                let old_y = (i as i32 - dy) as isize;
                if old_x >= 0 && old_x < WIDTH as isize && old_y >= 0 && old_y < HEIGHT as isize {
                    new_data[i][j] = self.data[old_y as usize][old_x as usize];
                }
            }
        }

        self.data = new_data;
    }
}

impl PhysicalWriter {
    pub fn render(&mut self, sx: usize, sy: usize, ex: usize, ey: usize) {
        //qemu_print(format!("Start Render... Now is {:?}\n", TIME.lock()).as_str());
        if sx < HEIGHT && sy < WIDTH && ex <= HEIGHT && ey <= WIDTH {
            if GL.read().len() == 0 { return; }
            let p_lock = GL.read();
            let lock = p_lock[p_lock.len() - 1].lock();
            let mut graph: Box<Vec<Vec<(Rgb888, bool)>>> = if lock.enable {
                Box::new(lock.data.clone())
            } else {
                let mut g = Box::new(lock.data.clone());
                for r in g.iter_mut() {
                    for p in r.iter_mut() {
                        p.1 = false;
                    }
                }
                g
            };
            drop(lock);
            for layer in (1..p_lock.len() - 1).rev() {
                let lock = p_lock[layer].lock();
                if !lock.enable { continue }
                let tomix = &lock.data;
                for x in sx..ex {
                    for y in sy..ey {
                        if !graph[x][y].1 && tomix[x][y].1 {
                            graph[x][y] = tomix[x][y]
                        }
                    }
                }
            }
            let tomix = &p_lock[0].lock().data;
            for x in sx..ex {
                for y in sy..ey {
                    // debugln!("{},{},{:?},{}",x,y,graph[x][y].0,graph[x][y].1);
                    graph[x][y].0 = if graph[x][y].1 { graph[x][y].0 } else { tomix[x][y].0 };
                }
            }
            for x in sx..ex {
                for y in sy..ey {
                    self.0.chars[x][y].write(graph[x][y].0);
                }
            }
        }
        //qemu_print(format!("Finish Render... Now is {:?}\n", TIME.lock()).as_str());
    }
}

pub fn resolve_32rgba(bmp_data: &[u8]) -> Vec<(usize, usize, Rgb888)> {
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
                //qemu_print(format!("{:?},{:?}", rgb_color, alpha).as_str());
                if alpha > 0.5 {
                    result.push((position.y as usize, position.x as usize, rgb_color))
                }
            }
        }
        Err(error) => {
            qemu_print(format!("{:?}\n", error).as_str());
        }
    }
    result
}

pub fn test_img() {
    GD.lock().display_rect(0, 0, 800, 600, rgb888!(0xFFFFFFu32));

    unimplemented!("请为我解除封印");
    // let lpld = include_bytes!("../assets/91527085_p0.bmp");
    // let cinea_os = include_bytes!("../assets/cinea-os.bmp");
    // GD.lock().display_img(0, 0, lpld);
    // GD.lock().display_img(400, 300, cinea_os);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    // 防止死锁
    interrupts::without_interrupts(|| {
        TEXT_WRITER.lock().write_fmt(args).unwrap();
    })
}


