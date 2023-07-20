//! This module provides functions for making system calls and low-level functions for interacting with the system call interface.
//!
//! The following functions are provided:
//!
//! - `log(buf: &[u8]) -> Option<usize>`: Write a log message to the system log.
//! - `log_debug(buf: &[u8]) -> Option<usize>`: Write a debug log message to the system log.
//! - `exit(code: ExitCode)`: Exit the current process with the specified exit code.
//! - `sleep(seconds: f64)`: Sleep for the specified number of seconds.
//! - `spawn(number: usize, args: &[&str]) -> Result<(), ExitCode>`: Spawn a new process with the specified number and arguments.
//! - `panic() -> usize`: Panic the kernel.
//! - `alloc(size: usize, align: usize) -> usize`: Allocate heap memory.
//! - `free(ptr: usize, size: usize, align: usize)`: Free heap memory.
//! - `stop_schedule()`: Stop scheduling for a while.
//! - `restart_schedule()`: Resume scheduling.
//!
//! The following low-level functions are provided:
//!
//! - `syscall0(n: usize) -> usize`: Make a system call with no arguments.
//! - `syscall1(n: usize, arg1: usize) -> usize`: Make a system call with one argument.
//! - `syscall2(n: usize, arg1: usize, arg2: usize) -> usize`: Make a system call with two arguments.
//! - `syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize`: Make a system call with three arguments.
//! - `syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize`: Make a system call with four arguments.
//!
//! # Examples
//!
//! ```
//! use crate::syscall::{self, log, log_debug, exit, spawn, panic, alloc, free, stop_schedule, restart_schedule};
//!
//! log(b"Hello, world!");
//! log_debug(b"Debug message");
//! exit(0);
//! sleep(1.0);
//! spawn(1, &["arg1", "arg2"]).unwrap();
//! panic();
//! let ptr = alloc(1024, 4);
//! free(ptr, 1024, 4);
//! stop_schedule();
//! restart_schedule();
//! ```
//!
//! # Safety
//!
//! The functions provided by this module are unsafe because they allow calling arbitrary system calls with arbitrary arguments.
//! It is the responsibility of the caller to ensure that the arguments are valid and that the system call is safe to make.
//! Additionally, the return value of the system call is not checked, so it is up to the caller to handle any errors that may occur.
//!
//! # Note
//!
//! The `log` function writes a message to the system log, which can be viewed using the `dmesg` command.
//! The `log_debug` function writes a message to the system log with a debug level, which can be filtered using the `loglevel` kernel parameter.
//! The `spawn` function takes a number and a slice of string arguments, and returns `Ok(())` if the process was successfully spawned, or `Err(ExitCode)` if an error occurred.
//! The `alloc` function takes a size and an alignment, and returns a pointer to the allocated memory.
//! The `free` function takes a pointer, a size, and an alignment, and frees the allocated memory.
//! The `stop_schedule` function stops scheduling for a while, and the `restart_schedule` function resumes scheduling.

use core::arch::asm;

use alloc::vec::Vec;

use crate::call::*;
use crate::ExitCode;
use crate::syscall;

pub fn log(buf: &[u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(LOG, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn log_debug(buf: &[u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(LOG, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn exit(code: ExitCode) {
    unsafe { syscall!(EXIT, code as usize) };
}

pub fn sleep(seconds: f64) {
    unsafe {
        syscall!(SLEEP, seconds.to_bits());
    }
}

pub fn spawn(number: usize, args: &[&str]) -> Result<(), ExitCode> {
    // log({ args.as_ptr() as usize }.to_string().as_bytes());
    let ptr_len_pair: Vec<(usize, usize)> = args
        .iter()
        .map(|arg| (arg.as_ptr() as usize, arg.len()))
        .collect();
    let (args_ptr, args_len, args_cap) = ptr_len_pair.into_raw_parts();
    let res = unsafe { syscall!(SPAWN, number, args_ptr as usize, args_len, args_cap) };
    if res == ExitCode::Success as usize {
        Ok(())
    } else {
        Err(ExitCode::from(res))
    }
}

pub fn panic() -> usize {
    unsafe { syscall!(PANIC) }
}

pub fn alloc(size: usize, align: usize) -> usize {
    unsafe { syscall!(ALLOC, size, align) }
}

pub fn free(ptr: usize, size: usize, align: usize) {
    unsafe { syscall!(FREE, ptr, size, align) };
}

pub fn stop_schedule() {
    unsafe { syscall!(NO_SCHE) };
}

pub fn restart_schedule() {
    unsafe { syscall!(CON_SCHE) };
}

/***
 * 发送系统调用
 */

#[doc(hidden)]
pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x80", in("rax") n,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x80", in("rax") n,
    in("rdi") arg1,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x80", in("rax") n,
    in("rdi") arg1, in("rsi") arg2,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x80", in("rax") n,
    in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x80", in("rax") n,
    in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("r8") arg4,
    lateout("rax") res
    );
    res
}
