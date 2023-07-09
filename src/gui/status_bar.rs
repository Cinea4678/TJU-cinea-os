use alloc::format;
use crate::graphic::{GD, GL, WIDTH};
use crate::io::time::get_raw_time;
use crate::rgb888;

pub fn show_status_bar() {
    let lock = GL.lock();
    lock[0].display_rect(0, 0, WIDTH, 18, rgb888!(0x90CAF9u32), 1.0);
    let time = get_raw_time();
    unsafe {
        lock[1].display_font_string(
            format!("{:02}:{:02}", time.hour, time.minute).as_str(),
            0, (WIDTH / 2) - ((16 * 5) / 2), 16.0, 16, rgb888!(0xffffffu32),
        );
        lock[1].display_font_string(
            "Cinea OS v1.0",
            0, 2, 16.0, 16, rgb888!(0xffffffu32),
        );
    };
}