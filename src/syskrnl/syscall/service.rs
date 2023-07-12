use crate::{println, syskrnl};
use crate::sysapi::proc::ExitCode;
use crate::syskrnl::proc::Process;

pub fn exit(code: ExitCode) -> ExitCode {
    syskrnl::proc::exit();
    code
}

pub fn sleep(seconds: f64) {
    syskrnl::time::sleep(seconds);
}

/// FIXME 在未来，要改正。现在是测试用途
pub fn spawn(number: usize, args_ptr: usize, args_len: usize) -> ExitCode {
    let subprocess = match number {
        // 0x00 => include_bytes!("../../dsk/bin/hello"),
        _ => {
            println!("spawn: invalid number");
            return ExitCode::OpenError;
        }
    };
    if let Err(code) = Process::spawn(subprocess, args_ptr, args_len) {
        code
    } else {
        ExitCode::Success
    }
}

pub fn log(msg: usize, len: usize) -> usize {
    let msg = unsafe { core::slice::from_raw_parts(msg as *const u8, len) };
    match core::str::from_utf8(msg) {
        Err(_) => {
            println!("log: invalid utf8 string");
            1
        }
        Ok(s) => {
            println!("{}", s);
            0
        }
    }
}