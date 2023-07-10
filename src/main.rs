// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use cinea_os::println;

use cinea_os::syskrnl::{allocator};
use cinea_os::syskrnl::graphic::enter_wide_mode;
use cinea_os::syskrnl::gui::init_gui;
use cinea_os::syskrnl::task::executor::Executor;
use cinea_os::syskrnl::task::keyboard::print_keypresses;
use cinea_os::syskrnl::task::Task;
use cinea_os::syskrnl::vga_buffer;

entry_point!(kernel_main);

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cinea_os::hlt_loop();
}

/// 内核主程序
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Launching Cinea's OS...\n");
    cinea_os::init();

    vga_buffer::print_something();

    println!("\n\nWaiting for initializing the heap memory...\n");

    use cinea_os::syskrnl::memory::BootInfoFrameAllocator;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.clone());
    let mut mapper = unsafe { cinea_os::syskrnl::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    println!("The OS is leaving VGA now...");

    enter_wide_mode(&mut mapper, &mut frame_allocator);
    init_gui();

    println!("\n\n\t\t万里之行，始于足下\n\n");

    println!("异步处理键盘输入测试：");

    let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();

    //cinea_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number)
}