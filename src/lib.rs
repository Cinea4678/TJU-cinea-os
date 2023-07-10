// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]

extern crate alloc;

use crate::syskrnl::io::mouse;

pub mod syskrnl;

pub fn init() {
    // 加载GDT
    syskrnl::gdt::init();

    // 加载中断和异常处理
    syskrnl::interrupts::init_idt();
    unsafe { syskrnl::interrupts::pics::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // 启用鼠标
    // mouse::init_mouse();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}