#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use cinea_os::sysapi::syscall;
use cinea_os::{entry_point, sysapi, STDOUT};
use ufmt::uwriteln;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    uwriteln!(STDOUT.lock(), "我是Shell（伪），以后操作系统就只有我了。我正在子进程运行。").unwrap();
    uwriteln!(STDOUT.lock(), "进程调度测试。（基础）").unwrap();
    uwriteln!(STDOUT.lock(), "现在启动第一个子进程。").unwrap();
    syscall::spawn(1, &[String::from("我是子进程A").as_str()]).expect("子进程启动失败");
    syscall::spawn(1, &[String::from("我是子进程B").as_str()]).expect("子进程启动失败");
    
    uwriteln!(STDOUT.lock(), "测试完成").unwrap();

    loop {
        syscall::sleep(1.0);
    }
}
