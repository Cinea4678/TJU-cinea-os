// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]

extern crate alloc;

use crate::syskrnl::io::mouse;

pub mod syskrnl;
pub mod sysapi;

pub fn init() {
    // 加载GDT
    syskrnl::gdt::init();

    // 加载中断和异常处理
    syskrnl::interrupts::init_idt();
    unsafe { syskrnl::interrupts::pics::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // 启用各类IO设备
    debugln!("Start timer");
    syskrnl::time::init();
    syskrnl::task::keyboard::init();
    // mouse::init_mouse();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}