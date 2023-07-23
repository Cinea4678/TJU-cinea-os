// src/lib.rs

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]
#![feature(exclusive_range_pattern)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};

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

#[cfg(test)]
entry_point!(kernel_main);

#[cfg(test)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    #[cfg(test)]
    test_main();

    hlt_loop()
}

/// 这个函数将在panic时被调用
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{:?}", info);
    hlt_loop();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}