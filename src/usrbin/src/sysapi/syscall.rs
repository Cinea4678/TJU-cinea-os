use alloc::vec::Vec;

use crate::sysapi::call::*;
use crate::sysapi::proc::ExitCode;
use crate::syscall;
use core::arch::asm;

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

pub fn alloc(size: usize, align: usize) -> usize {
    unsafe { syscall!(ALLOC, size, align) }
}

pub fn free(ptr: usize, size: usize, align: usize) {
    unsafe { syscall!(FREE, ptr, size, align) };
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
