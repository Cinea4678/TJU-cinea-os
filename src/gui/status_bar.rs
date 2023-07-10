use alloc::format;
use crate::graphic::{GD, GL, WIDTH};
use crate::io::time::get_raw_time;
use crate::rgb888;

pub fn show_status_bar() {
    GL.read()[0].lock().display_rect(0, 0, WIDTH, 18, rgb888!(0x37474Fu32));
    let time = get_raw_time();
    unsafe {
        GL.read()[1].lock().display_font_string(
            format!("{:02}:{:02}", time.hour, time.minute).as_str(),
            0, (WIDTH / 2) - ((16 * 5) / 2), 16.0, 16, rgb888!(0xffffffu32),
        );
        GL.read()[1].lock().display_font_string(
            "Cinea OS v1.0",
            0, 2, 16.0, 16, rgb888!(0xffffffu32),
        );
    };
}