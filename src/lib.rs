// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]

extern crate alloc;

pub mod interrupts;
pub mod vga_buffer;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod graphic;
pub mod gui;
pub mod io;

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