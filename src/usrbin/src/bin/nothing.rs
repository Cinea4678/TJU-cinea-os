#![no_std]
#![no_main]

extern crate alloc;

use cinea_os_userspace::{entry_point, sysapi};

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    return;
}
