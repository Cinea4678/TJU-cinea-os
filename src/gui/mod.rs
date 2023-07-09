use crate::graphic::{GD, GL, HEIGHT, WIDTH, Writer};
use crate::gui::cursor::display_cursor_first_time;
use crate::gui::status_bar::show_status_bar;
use crate::rgb888;

pub mod status_bar;
mod cursor;

/// 图层规则（暂定）
///
/// -1：鼠标
/// -2：留用
/// -3：留用
/// .....
/// 1: Console
/// 0: 背景
///
pub fn init_gui() {
    /// 创建0号图层和1号图层
    GL.lock().push(Writer::new());
    GL.lock().push(Writer::new());
    show_command_area();
    show_status_bar();
    /// 创建-3、-2和-1号图层
    GL.lock().push(Writer::new());
    GL.lock().push(Writer::new());
    GL.lock().push(Writer::new());
    display_cursor_first_time(HEIGHT / 2, WIDTH / 2);

    GD.lock().render(0, 0, HEIGHT, WIDTH);
}

fn show_command_area() {
    GL.lock()[0].display_rect(0, 0, WIDTH, HEIGHT, rgb888!(0x006699u32), 1.0);
}