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
    uwriteln!(STDOUT.lock(), "进程切换测试。").unwrap();
    uwriteln!(STDOUT.lock(), "现在进入子进程。").unwrap();
    syscall::spawn(0, &[String::from("Magical World!").as_str()]).expect("子进程未成功退出");
    uwriteln!(
        STDOUT.lock(),
        "子进程已经安全退出，CPU成功回到父进程（也在Ring3）"
    )
    .unwrap();
    loop {
        syscall::sleep(1.0);
    }
}
