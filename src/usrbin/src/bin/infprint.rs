#![no_std]
#![no_main]

extern crate alloc;

use cinea_os::{entry_point, STDOUT, sysapi};
use ufmt::uwriteln;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

fn main(args: &[&str]) {
    if args.len() > 0 {
        loop {
            uwriteln!(STDOUT.lock(),"{}",args[0]).unwrap();
            for _ in 0..100000 {  // 土法sleep
            }
        }
    }
}
