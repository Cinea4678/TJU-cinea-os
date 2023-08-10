use core::sync::atomic::{AtomicBool, Ordering};

use crate::syskrnl::graphic::GL;
use crate::syskrnl::gui::cursor::MOUSE_CURSOR;
use crate::syskrnl::gui::status_bar::show_status_bar;

pub mod status_bar;
pub mod cursor;
pub mod panic;
pub mod window;

pub use window::WINDOW_MANAGER;

pub static RENDER_OK: AtomicBool = AtomicBool::new(false);

/// 图层规则（暂定）
///
/// -1：鼠标
/// .....
/// 1: Console
/// 0: 背景
///
pub fn init() {
    GL.read()[0].lock().enable = true;
    GL.read()[1].lock().enable = true;
    GL.read()[2].lock().enable = true;
    GL.read()[4].lock().enable = true;
    show_command_area();
    show_status_bar();
    MOUSE_CURSOR.lock().print_manually();
    
    window::init();

    RENDER_OK.store(true, Ordering::Relaxed);
}

fn show_command_area() {
    //GL.read()[0].lock().display_rect(0, 0, WIDTH, HEIGHT, rgb888!(0x006699u32));

    let background_img = include_bytes!("../../../assets/OS_background.bmp");
    GL.read()[0].lock().display_img(0,0,background_img);
}