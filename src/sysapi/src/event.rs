use core::arch::asm;

use crate::event_call;

pub const KEYBOARD_INPUT: usize = 0x00;
pub const SLEEP_WAKEUP: usize = 0x01;

pub fn sleep(million_seconds: usize) {
    unsafe { event_call!(SLEEP_WAKEUP, million_seconds); }
}

#[macro_export]
macro_rules! event_call {
    ($n:expr) => (
        $crate::event::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::event::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::event::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::event::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => (
        $crate::event::syscall4(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize, $a4 as usize));
}

/***
 * 发送系统调用
 */

#[doc(hidden)]
pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x82", in("rax") n,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x82", in("rax") n,
    in("rdi") arg1,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x82", in("rax") n,
    in("rdi") arg1, in("rsi") arg2,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x82", in("rax") n,
    in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
    lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let res: usize;
    asm!(
    "int 0x82", in("rax") n,
    in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("r8") arg4,
    lateout("rax") res
    );
    res
}