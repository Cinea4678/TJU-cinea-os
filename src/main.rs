// src/main.rs

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有Rust层级的入口点
#![feature(abi_x86_interrupt)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};

use cinea_os::{debugln, println, syskrnl};

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

    cinea_os::init(boot_info);

    println!("\n\n\t\t万里之行，始于足下\n\t\t道阻且长，行则将至\n");

    println!("系统uptime：{:.5} s", syskrnl::time::get_uptime());

    // let mut executor = Executor::new();
    // //executor.spawn(Task::new(example_task()));
    // executor.spawn(Task::new(print_keypresses()));
    // executor.run();

    println!("我是内核，我即将启动用户进程并将CPU调整到环三！");
    debugln!("B-A");
    let subp = include_bytes!("../dsk/bin/hello");
    debugln!("B-B");
    let args: Vec<&str> = vec![];
    let res = syskrnl::proc::Process::spawn(subp, args.as_ptr() as usize, 0);
    println!("用户进程已退出，是否退出成功：{}\n顺便一提，我是系统内核进程，看到这条消息说明我在环零运行！", res.is_ok());

    cinea_os::hlt_loop();
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

