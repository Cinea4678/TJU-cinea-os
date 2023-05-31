// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Translate;
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

    use x86_64::VirtAddr;
    use  cinea_os::memory;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.clone());
    // new: initialize a mapper
    let mapper = unsafe { memory::init(phys_mem_offset) };
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset.clone(),
    ];
    for &address in &addresses {
        let virt = VirtAddr::new(address.clone());
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    cinea_os::hlt_loop();
}
