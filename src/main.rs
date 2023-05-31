// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Page;
use x86_64::structures::paging::Translate;
use x86_64::VirtAddr;
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
fn kernel_main(boot_info: &'static BootInfo) -> ! {

    println!("Loading Cinea's OS...\n");
    cinea_os::init();

    vga_buffer::print_something();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.clone());
    let mut mapper = unsafe{cinea_os::memory::init(phys_mem_offset)};
    let mut frame_allocator = cinea_os::memory::EmptyFrameAllocator;
    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    cinea_os::memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    cinea_os::hlt_loop();
}
