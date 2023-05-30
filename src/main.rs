// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点

mod vga_buffer;

use core::panic::PanicInfo;
use crate::vga_buffer::Writer;

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Every smallest dream matters.";

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {

    vga_buffer::print_something();

    loop {}
}
