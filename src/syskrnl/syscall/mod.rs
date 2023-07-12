use core::arch::asm;

/// 系统调用
///
/// 2023/7/11，怀着激动的心情，创建这个mod
///

mod service;

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

/// 打印日志 (2): a0-msg, a1-len
pub const LOG:      usize = 0xC;

pub fn dispatcher(syscall_id: usize, arg1: usize, arg2: usize, _arg3: usize, _arg4: usize) -> usize {
    match syscall_id {
        EXIT => unimplemented!(),
        SPAWN => unimplemented!(),
        READ => unimplemented!(),
        WRITE => unimplemented!(),
        OPEN => unimplemented!(),
        CLOSE => unimplemented!(),
        INFO => unimplemented!(),
        DUP => unimplemented!(),
        DELETE => unimplemented!(),
        STOP => unimplemented!(),
        SLEEP => unimplemented!(),
        LOG => service::log(arg1, arg2),
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