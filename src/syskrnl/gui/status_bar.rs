use alloc::format;
use crate::syskrnl::graphic::{GD, GL, WIDTH};
use crate::syskrnl::time::get_raw_time;
use crate::rgb888;
use crate::syskrnl::io;

pub fn show_status_bar() {
    GL.read()[0].lock().display_rect(0, 0, WIDTH, 18, rgb888!(0x37474Fu32));
    unsafe {
        GL.read()[1].lock().display_font_string(
            "Cinea OS v1.0",
            0, 2, 16.0, 16, rgb888!(0xffffffu32),
        );
    };
}

pub fn update_status_bar_time(show_colon: bool) {
    if io::VIDEO_MODE.lock().is_text() {
        return;
    }

    let time = get_raw_time();

    let time_str = if show_colon {
        format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second)
    } else {
        format!("{:02} {:02} {:02}", time.hour, time.minute, time.second)
    };

    unsafe {
        let p_lock = GL.read();
        let mut lock = p_lock[1].lock();
        lock.clear_rect(0, 330, 128, 20);
        // lock.clear_rect(200, 270, 256, 35);
        lock.display_font_string(
            time_str.as_str(), 0, (WIDTH / 2) - ((12 * 8) / 2), 16.0, 16, rgb888!(0xffffffu32),
        );
        // lock.display_font_string(
        //     time_str.as_str(), 200, (WIDTH / 2) - ((24 * 8) / 2), 32.0, 32, rgb888!(0xffffffu32),
        // );
        drop(lock);
        drop(p_lock);
        GD.lock().render(0, 330, 20, 450);
        // GD.lock().render(200, 270, 235, 530);
    }
}