#![no_std]
#![no_main]

extern crate alloc;

use cinea_os::entry_point;
use cinea_os::sysapi::syscall;

entry_point!(main);

fn main(_args: &[&str]) {
    syscall::log("我是Shell（伪），我正在子进程运行。".as_bytes());
    syscall::log("进程切换测试。".as_bytes());
    syscall::log("现在进入子进程。".as_bytes());
    syscall::spawn(0, ["Magical World!"].as_slice()).expect("子进程未成功退出");
    syscall::log("子进程已经安全退出，CPU成功回到父进程（也在Ring3）".as_bytes());
    loop {
        syscall::sleep(1.0);
    }
}