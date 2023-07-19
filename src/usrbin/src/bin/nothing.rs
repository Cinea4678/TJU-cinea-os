#![no_std]
#![no_main]

extern crate alloc;

use cinea_os_sysapi::{allocator, entry_point};

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn main(_args: &[&str]) {
    return;
}
