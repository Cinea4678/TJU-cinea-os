#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
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
        syscall::log(b"Hello World From User-Space!\n");
    }
}