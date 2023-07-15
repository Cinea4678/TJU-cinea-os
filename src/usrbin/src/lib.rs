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
use alloc::string::String;
use ufmt::uWrite;

pub struct StdWriter;

impl uWrite for StdWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        sysapi::syscall::log(s.as_bytes());
        Ok(())
    }
}

pub struct StringWriter{
    value: String
}

impl uWrite for StringWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.value += s;
        Ok(())
    }
}

impl StringWriter {
    pub fn new()->Self{
        Self { value: String::new() }
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }
}

pub fn without_schedule<F>(mut function: F) where F:FnMut() {
    sysapi::syscall::stop_schedule();
    function();
    sysapi::syscall::restart_schedule();
}