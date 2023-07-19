#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]
#![feature(vec_into_raw_parts)]

extern crate alloc;

extern crate cinea_os_sysapi;

#[macro_use]
pub mod std;

pub fn without_schedule<F>(mut function: F) where F:FnMut() {
    cinea_os_sysapi::syscall::stop_schedule();
    function();
    cinea_os_sysapi::syscall::restart_schedule();
}