#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;

use cinea_os::entry_point;
use cinea_os::sysapi::syscall;

entry_point!(main);

fn main(args: &[&str]) {
    let b = format!("Format test").clone();
    syscall::log_spc({
        let c = b.as_bytes();
        c
    });
    if args.len() > 0 {
        let hello_world_str = format!("Hello, {}", args[0]);
        syscall::log(hello_world_str.clone().as_bytes());
    } else {
        syscall::log(b"\nHello World From User-Space!\n");
    }
}