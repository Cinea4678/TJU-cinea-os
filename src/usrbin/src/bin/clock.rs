#![no_std]
#![no_main]

extern crate alloc;

use ufmt::uwrite;

use cinea_os_sysapi::{allocator, entry_point, gui, rgb888};
use cinea_os_sysapi::event::{GUI_EVENT_EXIT, GUI_EVENT_TIME_UPDATE, register_time_update, wait_gui_event};
use cinea_os_sysapi::gui::{load_font, remove_window_gui, WINDOW_CONTENT_HEIGHT, WINDOW_CONTENT_WIDTH, WindowWriter};
use cinea_os_sysapi::time::get_datetime;
use cinea_os_userspace::debugln;
use cinea_os_userspace::std::StringWriter;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn update_display_time(window_instance: &mut WindowWriter) {
    let dt = get_datetime();

    let mut minute = StringWriter::new();
    if dt.time.min < 10 { uwrite!(minute, "0{}", dt.time.min).unwrap() } else { uwrite!(minute, "{}", dt.time.min).unwrap() }

    let mut h_m = StringWriter::new();
    uwrite!(h_m, "{}:{}", dt.time.hour, minute.value().as_str()).unwrap();

    let mut sec = StringWriter::new();
    if dt.time.sec < 10 { uwrite!(sec, "0{}", dt.time.sec).unwrap() } else { uwrite!(sec, "{}", dt.time.sec).unwrap() }

    let mut date = StringWriter::new();
    uwrite!(date, "{}/ {}/ {}", dt.date.year, dt.date.month, dt.date.day).unwrap();

    window_instance.clear_rect(0, 0, WINDOW_CONTENT_HEIGHT, WINDOW_CONTENT_WIDTH);
    window_instance.display_font_string(h_m.value().as_str(), "Handjet", 20, 10, 52.0, 52, rgb888!(0));
    window_instance.display_font_string(sec.value().as_str(), "Handjet", 36, 105, 36.0, 36, rgb888!(0));
    window_instance.display_font_string(date.value().as_str(), "Handjet", 80, 10, 36.0, 36, rgb888!(0));
}

fn main(_args: &[&str]) {
    if !load_font("Handjet", "/sys/ast/Handjet-Light.ttf") {
        panic!("Load font failed")
    }
    let mut window_instance: WindowWriter = gui::init_window_gui("时间", rgb888!(0xffffffu32)).expect("获取窗口实例失败");
    register_time_update(); // 订阅时间更新事件
    update_display_time(&mut window_instance);
    loop {
        let (code, _arg0, _arg1, _arg2) = wait_gui_event();
        match code {
            GUI_EVENT_EXIT => {
                remove_window_gui(window_instance);
                break;
            }
            GUI_EVENT_TIME_UPDATE => {
                update_display_time(&mut window_instance);
            }
            _ => {}
        }
    }
}
