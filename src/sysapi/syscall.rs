use alloc::format;

use crate::sysapi::proc::ExitCode;
use crate::syscall;
use crate::syskrnl::syscall::call::*;

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

pub fn log_spc(buf: &[u8]) -> Option<usize> {
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
    unsafe { syscall!(SLEEP, seconds.to_bits()); }
}

pub fn spawn(number: usize, args: &[&str]) -> Result<(), ExitCode> {
    let args_ptr = args.as_ptr() as usize;
    let args_len = args.len();
    let res = unsafe {
        syscall!(SPAWN, number, args_ptr, args_len)
    };
    if res == ExitCode::Success as usize { Ok(()) } else { Err(ExitCode::from(res)) }
}