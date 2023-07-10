// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use cinea_os::{allocator, println};
use cinea_os::graphic::enter_wide_mode;
use cinea_os::gui::init_gui;
use cinea_os::io::qemu::qemu_print;
use cinea_os::vga_buffer;

entry_point!(kernel_main);

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cinea_os::hlt_loop();
}

/// 内核主程序
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Loading Cinea's OS...\n");
    cinea_os::init();

    vga_buffer::print_something();

    use cinea_os::memory::BootInfoFrameAllocator;
    qemu_print("A\n");

    println!("\n\nWaiting for initializing the heap memory...\n");


    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.clone());
    let mut mapper = unsafe { cinea_os::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    qemu_print("B\n");

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");
    qemu_print("C\n");

    qemu_print("The OS is leaving VGA now...\n");

    enter_wide_mode(&mut mapper, &mut frame_allocator);
    init_gui();

    println!("\n\n\t\t万里之行，始于足下");

    cinea_os::hlt_loop();
}
