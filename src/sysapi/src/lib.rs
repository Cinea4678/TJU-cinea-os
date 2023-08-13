//! This crate provides macros and functions for both kernel and userspace.
//!
//! Especially, this module provides a set of macros for making system calls.
//!
//! The macros provided are:
//!
//! - `syscall0(n: usize) -> usize`: Make a system call with no arguments.
//! - `syscall1(n: usize, a1: usize) -> usize`: Make a system call with one argument.
//! - `syscall2(n: usize, a1: usize, a2: usize) -> usize`: Make a system call with two arguments.
//! - `syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> usize`: Make a system call with three arguments.
//! - `syscall4(n: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> usize`: Make a system call with four arguments.
//!
//! # Examples
//!
//! ```
//! let result = syscall0(SYS_getpid);
//! println!("PID: {}", result);
//! ```
//!
//! # Safety
//!
//! These macros are unsafe because they allow calling arbitrary system calls with arbitrary arguments.
//! It is the responsibility of the caller to ensure that the arguments are valid and that the system call is safe to make.
//! Additionally, the return value of the system call is not checked, so it is up to the caller to handle any errors that may occur.

#![no_std] // 不链接Rust标准库
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(atomic_bool_fetch_not)]
#![feature(naked_functions)]
#![feature(vec_into_raw_parts)]
#![feature(panic_info_message)]
#![feature(let_chains)]

extern crate alloc;

#[macro_use]
pub mod call;

#[macro_use]
pub mod event;

pub mod allocator;
pub mod fs;
pub mod syscall;
pub mod time;
pub mod stdin;
pub mod window;

/// 进程退出代码
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success = 0,
    Failure = 1,
    UsageError = 64,
    DataError = 65,
    OpenError = 128,
    ReadError = 129,
    ExecError = 130,
    PageFaultError = 200,
    ShellExit = 255,
}

impl From<usize> for ExitCode {
    fn from(code: usize) -> Self {
        match code {
            0 => ExitCode::Success,
            64 => ExitCode::UsageError,
            65 => ExitCode::DataError,
            128 => ExitCode::OpenError,
            129 => ExitCode::ReadError,
            130 => ExitCode::ExecError,
            200 => ExitCode::PageFaultError,
            255 => ExitCode::ShellExit,
            _ => ExitCode::Failure,
        }
    }
}

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            // use alloc::format;
            // $crate::sysapi::syscall::log(format!("An exception occurred!\n{}", info).as_bytes());

            $crate::syscall::panic(info);
            loop {}
        }

        #[export_name = "_start"]
        pub unsafe extern "sysv64" fn __impl_start(args_ptr: u64, args_len: usize) {
            let args = core::slice::from_raw_parts(args_ptr as *const _, args_len);
            let f: fn(&[&str]) = $path;
            f(args);
            $crate::syscall::exit($crate::ExitCode::Success);
        }
    };
}

#[macro_export]
macro_rules! rgb888 {
    ($num:expr) => {
        embedded_graphics::pixelcolor::Rgb888::new(($num>>16) as u8,($num>>8) as u8,$num as u8)
    };
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => {
        $crate::syscall::syscall0($n as usize)
    };
    ($n:expr, $a1:expr) => {
        $crate::syscall::syscall1($n as usize, $a1 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr) => {
        $crate::syscall::syscall2($n as usize, $a1 as usize, $a2 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::syscall::syscall3($n as usize, $a1 as usize, $a2 as usize, $a3 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        $crate::syscall::syscall4(
            $n as usize,
            $a1 as usize,
            $a2 as usize,
            $a3 as usize,
            $a4 as usize,
        )
    };
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
