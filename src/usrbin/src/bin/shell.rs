#![no_std]
#![no_main]

extern crate alloc;

use cinea_os_userspace::{sysapi, entry_point, print};

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

fn main(_args: &[&str]) {
    print!("Cinea OS Shell v0.1.0");

    loop {}
}
