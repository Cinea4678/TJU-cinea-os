#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;

use ufmt::uwriteln;

use cinea_os_sysapi::{allocator, entry_point, syscall};
use cinea_os_userspace::std::StdWriter;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn main(args: &[&str]) {
    let mut stdout = StdWriter;
    if args.len() > 0 {
        uwriteln!(
            stdout,
            "Hello, {}",
            args[0]
        )
        .unwrap();
    } else {
        uwriteln!(stdout, "\nHello World From User-Space!\n").unwrap();
        syscall::log(format!("哥们就是用format，怎么了！{}", "哈哈").as_bytes());
    }
}
