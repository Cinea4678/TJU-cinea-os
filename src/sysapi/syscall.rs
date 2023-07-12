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

pub fn exit(code: ExitCode) {
    unsafe { syscall!(EXIT, code as usize) };
}