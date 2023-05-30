// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use cinea_os::interrupts::pics::PICS;
use cinea_os::println;
use cinea_os::vga_buffer;

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cinea_os::hlt_loop();
}

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {

    println!("Loading Cinea's OS...\n");
    cinea_os::init();

    vga_buffer::print_something();

    let level_4_table_pointer = 0xffff_ffff_ffff_f000 as *const u64;
    for i in 0..10 {
        let entry = unsafe { *level_4_table_pointer.offset(i) };
        println!("Entry {}: {:#x}", i, entry);
    }

    cinea_os::hlt_loop();
}
