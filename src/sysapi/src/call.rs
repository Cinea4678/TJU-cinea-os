//! This module provides constants for system call numbers.
//!
//! The following system calls are provided:
//!
//! - `EXIT`: Exit the current process.
//! - `SPAWN`: Spawn a new process.
//! - `READ`: Read from a file descriptor.
//! - `WRITE`: Write to a file descriptor.
//! - `OPEN`: Open a file.
//! - `CLOSE`: Close a file descriptor.
//! - `INFO`: Get information about a file.
//! - `DUP`: Duplicate a file descriptor.
//! - `DELETE`: Delete a file.
//! - `STOP`: Stop the current process.
//! - `SLEEP`: Sleep for a specified number of milliseconds.
//! - `LOG`: Print a log message.
//! - `ALLOC`: Allocate heap memory.
//! - `FREE`: Free heap memory.
//! - `PANIC`: Panic the kernel.
//! - `NO_SCHE`: Stop scheduling for a while.
//! - `CON_SCHE`: Resume scheduling.
//!
//! # Examples
//!
//! ```
//! use crate::syscall::{self, EXIT};
//!
//! syscall::syscall1(EXIT, 0);
//! ```
//!
//! # Safety
//!
//! These constants are safe to use as system call numbers, but the system calls themselves are unsafe.
//! It is the responsibility of the caller to ensure that the arguments passed to the system calls are valid and that the system calls are safe to make.
//! Additionally, the return values of the system calls are not checked, so it is up to the caller to handle any errors that may occur.

/// exit the process
pub const EXIT:     usize = 0x1;
pub const SPAWN:    usize = 0x2;
pub const READ:     usize = 0x3;
pub const WRITE:    usize = 0x4;
pub const OPEN:     usize = 0x5;
pub const CLOSE:    usize = 0x6;
pub const INFO:     usize = 0x7;
pub const DUP:      usize = 0x8;
pub const DELETE:   usize = 0x9;
pub const STOP:     usize = 0xA;
pub const SLEEP:    usize = 0xB;
/// print logs (2): a0-msg, a1-len
pub const LOG:      usize = 0xC;
/// alloc heap memories (2): a0-size a1-align ret-ptr(usize)
pub const ALLOC:    usize = 0xD;
/// free heap memories a0-ptr a1-size a2-align
pub const FREE: usize = 0xE;
pub const PANIC: usize = 0xF;
/// stop schedules for a while
pub const NO_SCHE: usize = 0x10;
/// resume schedule
pub const CON_SCHE: usize = 0x11;