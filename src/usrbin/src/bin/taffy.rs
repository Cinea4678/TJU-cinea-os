#![no_std]
#![no_main]

extern crate alloc;

use cinea_os_sysapi::{allocator, entry_point, rgb888, window};
use cinea_os_sysapi::event::sleep;
use cinea_os_sysapi::font::load_font;
use cinea_os_userspace::print;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    print!("Taffy进程已启动\n");
    let mut window_instance = window::init_window_gui("测试 GUI 窗口渲染", rgb888!(0xffffffu32)).expect("获取窗口实例失败");
    load_font("Vonwaon", "/sys/ast/VonwaonBitmap-16px.ttf").expect("Load Font Failed");
    window_instance.display_rect(20, 20, 20, 20, rgb888!(0xff0000u32));
    unsafe {
        window_instance.display_font_string("关注永雏塔菲喵", "Vonwaon", 10, 10, 16.0, 16, rgb888!(0xff0000u32));
    }

    loop {
        print!("关注永雏塔菲喵\n");
        sleep(1000);
    }
}
