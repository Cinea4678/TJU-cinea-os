use crate::syskrnl::graphic::{GD, GL, HEIGHT, WIDTH};
use crate::syskrnl::gui::cursor::display_cursor_first_time;
use crate::syskrnl::gui::status_bar::show_status_bar;
use crate::syskrnl::io::qemu::qemu_print;

pub mod status_bar;
mod cursor;

/// 图层规则（暂定）
///
/// -1：鼠标
/// .....
/// 1: Console
/// 0: 背景
///
pub fn init_gui() {
    qemu_print("D\n");
    GL.read()[0].lock().enable = true;
    GL.read()[1].lock().enable = true;
    GL.read()[2].lock().enable = true;
    qemu_print("E\n");
    show_command_area();
    show_status_bar();
    display_cursor_first_time(HEIGHT / 2, WIDTH / 2);

    GD.lock().render(0, 0, HEIGHT, WIDTH);
}

fn show_command_area() {
    //GL.read()[0].lock().display_rect(0, 0, WIDTH, HEIGHT, rgb888!(0x006699u32));

    let background_img = include_bytes!("../../../assets/OS_background.bmp");
    GL.read()[0].lock().display_img(0,0,background_img);
}