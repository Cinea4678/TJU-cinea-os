#![no_std]
#![no_main]

extern crate alloc;

use cinea_os::{entry_point, sysapi};
use cinea_os::sysapi::syscall;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: sysapi::allocator::UserProcAllocator = sysapi::allocator::UserProcAllocator;

use core::convert::Infallible;
use ufmt::{uWrite, uwriteln};

struct MyWriter;

impl uWrite for MyWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        syscall::log_spc(s.as_bytes());
        Ok(())
    }
}

fn main(args: &[&str]) {
    let mut stdout = MyWriter;
    if args.len() > 1 {
        uwriteln!(stdout, "Hello, {}", args[0]).unwrap();
    } else {
        uwriteln!(stdout, "\nHello World From User-Space!\n").unwrap();
    }
}