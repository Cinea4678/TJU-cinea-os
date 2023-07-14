#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::Ordering;

use alloc::format;
use cinea_os::sysapi::syscall;
use cinea_os::{entry_point, sysapi, STDOUT, TEST_INT};
use ufmt::uwriteln;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(args: &[&str]) {
    if args.len() > 0 {
        uwriteln!(
            STDOUT.lock(),
            "Hello, {}, {}",
            args[0],
            TEST_INT.fetch_add(1, Ordering::Relaxed)
        )
        .unwrap();
    } else {
        uwriteln!(STDOUT.lock(), "\nHello World From User-Space!\n").unwrap();
        syscall::log(format!("哥们就是用format，怎么了！{}", "哈哈").as_bytes());
    }
}
