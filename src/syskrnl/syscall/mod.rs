use core::arch::asm;

use call::*;

use crate::debugln;
use crate::sysapi::proc::ExitCode;

/// 系统调用
///
/// 2023/7/11，怀着激动的心情，创建这个mod
///

mod service;
pub mod call;

pub fn dispatcher(syscall_id: usize, arg1: usize, arg2: usize, arg3: usize, _arg4: usize) -> usize {
    match syscall_id {
        EXIT => service::exit(ExitCode::from(arg1)) as usize,
        SPAWN => service::spawn(arg1, arg2, arg3) as usize,
        READ => unimplemented!(),
        WRITE => unimplemented!(),
        OPEN => unimplemented!(),
        CLOSE => unimplemented!(),
        INFO => unimplemented!(),
        DUP => unimplemented!(),
        DELETE => unimplemented!(),
        STOP => unimplemented!(),
        SLEEP => {
            service::sleep(f64::from_bits(arg1 as u64));
            0
        }
        LOG => {
            service::log(arg1, arg2)
        }
        _ => panic!("unknown syscall id: {}", syscall_id),
    }
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

/// 系统调用宏（最多四参数）
#[macro_export]
macro_rules! syscall {
    ($n:expr) => (
        $crate::syskrnl::syscall::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::syskrnl::syscall::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::syskrnl::syscall::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::syskrnl::syscall::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => (
        $crate::syskrnl::syscall::syscall4(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize, $a4 as usize));
}