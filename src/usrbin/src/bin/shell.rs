#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::{format, vec::Vec};

use cinea_os_sysapi::{allocator, entry_point, syscall::spawn};
use cinea_os_userspace::{print, std};

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

fn main(_args: &[&str]) {
    let nowdir = String::from("~");

    loop {
        print!("{} $ ", nowdir.as_str());

        loop {}
    }
}
