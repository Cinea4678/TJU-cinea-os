// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]
#![feature(exclusive_range_pattern)]

extern crate alloc;

use bootloader::BootInfo;

pub mod syskrnl;

/// 内核大小，暂定4MB
pub const KERNEL_SIZE: usize = 4 << 20; // 4 MB

pub fn init(bootinfo: &'static BootInfo) {
    // 加载GDT
    syskrnl::gdt::init();

    // 加载中断和异常处理
    syskrnl::interrupts::init_idt();
    unsafe { syskrnl::interrupts::pics::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    syskrnl::vga_buffer::print_something();

    // 加载内存
    println!("\n\nInitializing the memory...\n");
    syskrnl::memory::init(bootinfo);
    syskrnl::gui::init();

    // 启用各类IO设备
    syskrnl::io::ata::init();
    syskrnl::time::init();
    syskrnl::task::keyboard::init();
    syskrnl::task::mouse::init();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}