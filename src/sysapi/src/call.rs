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

use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;

use serde::{Deserialize, Serialize};

/// exit the process
pub const EXIT: usize = 0x1;
pub const SPAWN: usize = 0x2;
pub const INFO: usize = 0x7;
pub const DUP: usize = 0x8;
pub const DELETE: usize = 0x9;
pub const STOP: usize = 0xA;
pub const SLEEP: usize = 0xB;
/// print logs (2): a0-msg, a1-len
pub const LOG: usize = 0xC;
/// alloc heap memories (2): a0-size a1-align ret-ptr(usize)
pub const ALLOC: usize = 0xD;
/// free heap memories a0-ptr a1-size a2-align
pub const FREE: usize = 0xE;
pub const PANIC: usize = 0xF;
/// stop schedules for a while
pub const NO_SCHE: usize = 0x10;
/// resume schedule
pub const CON_SCHE: usize = 0x11;
pub const TEST_SERDE: usize = 0x12;
/// list files and directories in specified directory.
///
/// format: (2): a0-len,a1-postcarded FE ret-postcarded Vec-FE
///
/// *Not recommend for mannual use.*
pub const LIST: usize = 0x20;
pub const OPEN: usize = 0x21;
pub const CLOSE: usize = 0x22;
pub const WRITE_ALL: usize = 0x23;
pub const READ: usize = 0x24;
pub const WRITE_PATH: usize = 0x25;
pub const READ_PATH: usize = 0x26;
pub const CREATE_WINDOW: usize = 0x30;

#[derive(Debug, Serialize, Deserialize)]
pub struct SysCallResult {
    pub error: bool,
    pub error_code: usize,
    pub result_ptr: usize,
}

impl SysCallResult {
    pub fn error(code: usize) -> Self {
        Self {
            error: true,
            error_code: code,
            result_ptr: 0,
        }
    }

    pub fn success(result_ptr: usize) -> Self {
        Self {
            error: false,
            error_code: 0,
            result_ptr,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct _TestSerde {
    pub message: alloc::string::String,
    pub number: usize,
}

// impl uDebug for _TestSerde {
//     fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
//         where
//             W: ufmt::uWrite + ?Sized, {
//         f.debug_struct("TestSerdePass")?
//             .field("message", &self.message.as_bytes())?
//             .field("number", &self.number)?
//             .finish()?;
//         Ok(())
//     }
// }

pub fn syscall_serialized<T>(data: &T) -> usize where T: Serialize {
    let vecdata = postcard::to_allocvec(data).unwrap();
    let addr = vecdata.into_raw_parts();
    let addr_slice = vec![addr.0 as usize, addr.1, addr.2];
    let final_ptr = addr_slice.into_raw_parts().0;

    final_ptr as usize
}

fn copy_vec_to_ptr<T: Copy>(vec: &Vec<T>, ptr: *mut T) {
    unsafe {
        core::ptr::copy(vec.as_ptr(), ptr, vec.len());
    }
}

pub fn syscall_serialized_for_userspace<T, A>(data: &T, mut alloc_func: A) -> usize
    where T: Serialize, A: FnMut(Layout) -> *mut u8 {
    let vecdata = postcard::to_allocvec(data).unwrap();
    let layout = Layout::from_size_align(vecdata.len() * core::mem::size_of::<u8>(), core::mem::align_of::<u8>()).unwrap();
    let heap_addr = alloc_func(layout);
    copy_vec_to_ptr(&vecdata, heap_addr);

    let addr = vecdata.into_raw_parts();
    let addr_slice = vec![heap_addr as usize, addr.1, addr.2];
    let layout = Layout::from_size_align(3 * core::mem::size_of::<usize>(), core::mem::align_of::<usize>()).unwrap();
    let heap_addr = alloc_func(layout);
    copy_vec_to_ptr(&addr_slice, heap_addr as *mut usize);

    heap_addr as usize
}

pub fn syscall_deserialized_prepare(ptr: usize) -> Vec<u8> {
    let addr_slice = unsafe { Vec::from_raw_parts(ptr as *mut usize, 3, 3) };
    unsafe { Vec::from_raw_parts(addr_slice[0] as *mut u8, addr_slice[1], addr_slice[2]) }
}

pub fn syscall_deserialized<'de, T>(vec_data: &'de Vec<u8>) -> Result<T, postcard::Error> where T: Deserialize<'de> {
    postcard::from_bytes(vec_data.as_slice())
}

#[macro_export]
macro_rules! syscall_with_deserialize {
    ($($arg:tt)*) => {
        {
            let _ret = unsafe { $crate::syscall!($($arg)*) };
            let _ret_vec_data = $crate::call::syscall_deserialized_prepare(_ret);
            $crate::call::syscall_deserialized(&_ret_vec_data)
        }
    };
}

#[macro_export]
macro_rules! syscall_with_serdeser {
    ($call:expr,$obj:expr) => {
        {
            let _encoded = $crate::call::syscall_serialized(&$obj);
            let _ret = unsafe { $crate::syscall!($call, _encoded) };
            let _ret_vec_data = $crate::call::syscall_deserialized_prepare(_ret);
            $crate::call::syscall_deserialized(&_ret_vec_data)
        }
    };
}

#[macro_export]
macro_rules! syscall_with_serialize {
    ($call:expr,$obj:expr) => {
        {
            let _encoded = $crate::call::syscall_serialized(&$obj);
            unsafe { $crate::syscall!($call, _encoded) };
        }
    };
}