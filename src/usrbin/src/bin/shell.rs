#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use cinea_os::{entry_point, StdWriter, sysapi};
use cinea_os::sysapi::syscall;
use ufmt::uwriteln;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(args: &[&str]) {
    let mut stdout = StdWriter;
    uwriteln!(stdout, "我是Shell（伪），以后操作系统就只有我了。我正在子进程运行。").unwrap();
    uwriteln!(stdout, "进程调度测试。（基础）").unwrap();
    uwriteln!(stdout, "现在启动第一个子进程。").unwrap();
    uwriteln!(stdout, "args_ptr: {:#x}",args.as_ptr() as usize).unwrap();
    let _ = syscall::spawn(1, &[String::from("我是进程").as_str()]);
    let _ = syscall::spawn(1, &[String::from("我是另外一个进程").as_str()]);

    loop {}
}
