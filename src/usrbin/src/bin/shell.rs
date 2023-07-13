#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use core::arch::asm;
use core::ops::Add;
use cinea_os::{entry_point, sysapi};
use cinea_os::sysapi::syscall;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    syscall::log("我是Shell（伪），我正在子进程运行。".as_bytes());
    syscall::log("进程切换测试。".as_bytes());
    syscall::log("现在进入子进程。".as_bytes());
    // unsafe { asm!("int 3") }
    // let s1 = String::from("Magical ");
    // let s2 = String::from("World!");
    // let s3 = s1.add(s2.as_str());
    syscall::spawn(0, &[/*s3.as_str()*/]).expect("子进程未成功退出");
    syscall::log("子进程已经安全退出，CPU成功回到父进程（也在Ring3）".as_bytes());
    loop {
        syscall::sleep(1.0);
    }
}