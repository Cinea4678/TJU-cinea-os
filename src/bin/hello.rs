#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::arch::asm;
use x86::halt;
use cinea_os::sysapi::syscall;
use cinea_os::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() > 1 {
        // FIXME: This will result in a page fault exception for an address
        // that's already mapped to the kernel stack
        // 备注：暂时不管了，反正是测试用的
        syscall::log(format!("Hello, {}!\n", args[1]).as_bytes());
    } else {
        syscall::log(b"\nHello World From User-Space!\n");
        syscall::log("我是来自用户空间的进程，我通过触发0x80号中断来调用内核的指令！".as_bytes());
        syscall::log("\n接下来我要作死，尝试调用不可以调用的中断，不出意外的话下一行就是一般保护错误的提示了（笑）".as_bytes());
        unsafe {
            asm!(
            "int 0x10"
            );
            halt();
        }
    }
}