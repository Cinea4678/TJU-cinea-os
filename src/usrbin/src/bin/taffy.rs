#![no_std]
#![no_main]

extern crate alloc;

use cinea_os_sysapi::{allocator, entry_point, ExitCode, gui, rgb888};
use cinea_os_sysapi::event::{GUI_EVENT_EXIT, wait_gui_event};
use cinea_os_sysapi::fs::read_all_from_path;
use cinea_os_sysapi::gui::{load_font, remove_window_gui, WindowWriter};
use cinea_os_sysapi::syscall::exit;
use cinea_os_userspace::print;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    print!("Taffy进程已启动\n");
    if !load_font("Vonwaon", "/sys/ast/VonwaonBitmap-16px.ttf") { panic!("Load font failed") }
    let mut window_instance: WindowWriter = gui::init_window_gui("测试 GUI 窗口渲染", rgb888!(0xffffffu32)).expect("获取窗口实例失败");
    let taffy_img = read_all_from_path("/sys/ast/taffy.bmp").unwrap();
    let resolved_taffy_img = WindowWriter::resolve_img(taffy_img.as_slice()).unwrap();
    window_instance.display_resolved(0, 150, &resolved_taffy_img);
    window_instance.display_font_string("关注永雏塔菲喵", "Vonwaon", 10, 10, 16.0, 16, rgb888!(0xff0000u32));
    window_instance.display_font_string("关注永雏塔菲谢谢喵", "Vonwaon", 28, 10, 16.0, 16, rgb888!(0xff0000u32));
    loop {
        //print!("关注永雏塔菲喵\n");
        let (code, _arg0, _arg1, _arg2) = wait_gui_event();
        match code {
            GUI_EVENT_EXIT => {
                remove_window_gui(window_instance);
                break;
            }
            _ => {}
        }
    }
}
