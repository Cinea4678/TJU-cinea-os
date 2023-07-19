// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86::int;

use cinea_os::{debugln, println, syskrnl};
use cinea_os::syskrnl::task::executor::Executor;
use cinea_os::syskrnl::task::keyboard::print_keypresses;
use cinea_os::syskrnl::task::Task;

entry_point!(kernel_main);

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cinea_os::hlt_loop();
}

/// 内核主程序（0号进程）
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Launching Cinea's OS...\n");

    cinea_os::init(boot_info);

    println!("Cinea OS v1.0-dev by Cinea (Zhang Yao) cineazhan@icloud.com");
    println!("System Uptime：{:.5} s\n", syskrnl::time::uptime());

    //println!("我是内核，我即将启动用户进程并将CPU调整到环三！");

    let subp = include_bytes!("../dsk/bin/shell");
    let args: Vec<&str> = vec![];
    let mut flag = 0;
    loop {
        unsafe { int!(0x81) };
        debugln!("I come back with flag=={}", flag);
        #[allow(unused)]
        if flag > 0 { break; } else { flag = 1; } // 确保不会无尽循环启动shell
        syskrnl::proc::Process::spawn(subp, args.as_ptr() as usize, 0, 0).unwrap();
        panic!("The process is Cracked.");
    }

    // 0号进程继续它的工作：Handle键盘输入。
    let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();
}

// async fn async_number() -> u32 {
//     syskrnl::time::nanowait(100000);
//     42
// }
//
// async fn example_task() {
//     loop {
//         let number = async_number().await;
//         println!("async number: {}", number)
//     }
// }

