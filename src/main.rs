// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use cinea_os::interrupts::pics::PICS;
use cinea_os::println;
use cinea_os::vga_buffer;

entry_point!(kernel_main);

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cinea_os::hlt_loop();
}

/// 内核主程序
fn kernel_main(_boot_info: &'static BootInfo) -> ! {

    println!("Loading Cinea's OS...\n");
    cinea_os::init();

    vga_buffer::print_something();

    use x86_64::structures::paging::PageTable;
    let level_4_table_ptr = 0xffff_ffff_ffff_f000 as *const PageTable;
    let level_4_table = unsafe {&*level_4_table_ptr};
    for i in 0..10 {
        println!("Entry {}: {:?}", i, level_4_table[i]);
    }

    cinea_os::hlt_loop();
}
