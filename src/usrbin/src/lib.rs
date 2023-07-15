#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]
#![feature(vec_into_raw_parts)]

extern crate alloc;

#[macro_use]
pub mod sysapi;

use core::convert::Infallible;
use lazy_static::lazy_static;
use spin::Mutex;
use ufmt::uWrite;

pub struct MyWriter;

impl uWrite for MyWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        sysapi::syscall::log(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    pub static ref STDOUT: Mutex<MyWriter> = Mutex::new(MyWriter);
}
