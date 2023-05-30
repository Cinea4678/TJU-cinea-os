// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod interrupts;
pub mod vga_buffer;
pub mod gdt;

pub fn init() {
    // 加载GDT
    gdt::init();

    // 加载中断和异常处理
    interrupts::init_idt();
    unsafe { interrupts::pics::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}