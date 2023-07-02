// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;

use cinea_os::{allocator, println};
use cinea_os::graphic::{display_img, display_rect, enter_wide_mode};
use cinea_os::memory::graphic_support::create_graphic_memory_mapping;
use cinea_os::qemu::qemu_print;
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
    use cinea_os::allocator;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.clone());
    let mut mapper = unsafe { cinea_os::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    // let heap_value = Box::new(831);
    // println!("heap_value is at {:p}", heap_value);
    //
    // let mut vec = Vec::new();
    // for i in 0..500 {
    //     vec.push(i)
    // }
    // println!("vec at {:p}", vec.as_slice());
    //
    // let reference_counted = Rc::new(vec![1, 2, 3]);
    // let cloned_reference = reference_counted.clone();
    // println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    // core::mem::drop(reference_counted);
    // println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    qemu_print("The OS is leaving VGA now...\n");

    enter_wide_mode(&mut mapper, &mut frame_allocator);

    display_rect(0, 0, 800, 600, 0xFFFFFF);

    let lpld = include_bytes!("../assets/91527085_p0.bmp");
    let cinea_os = include_bytes!("../assets/cinea-os.bmp");
    display_img(0, 0, lpld);
    display_img(400, 300, cinea_os);


    cinea_os::hlt_loop();
}
