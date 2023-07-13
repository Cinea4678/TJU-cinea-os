#![no_std]
#![no_main]

extern crate alloc;

use cinea_os::entry_point;
use cinea_os::sysapi::syscall;

entry_point!(main);

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
    if args.len() > 0 {
        uwriteln!(stdout, "Hello, {}", args[0]).unwrap();
    } else {
        uwriteln!(stdout, "\nHello World From User-Space!\n").unwrap();
    }
}