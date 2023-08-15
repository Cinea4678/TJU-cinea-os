use core::arch::asm;

use crate::{event_call, syscall};
use crate::call::{GUI_SUBSCRIBE_KEYBOARD, GUI_SUBSCRIBE_TIME_UPDATE, REGISTER_TIMER};
use crate::syscall::log;

pub const KEYBOARD_INPUT: usize = 0x00;
pub const SLEEP_WAKEUP: usize = 0x01;
pub const GUI_PROGRAM: usize = 0x02;

pub fn sleep(million_seconds: usize) {
    unsafe { event_call!(SLEEP_WAKEUP, million_seconds); }
}

pub fn getch(display_back: bool) -> char {
    unsafe {
        let res = char::from_u32_unchecked(event_call!(KEYBOARD_INPUT) as u32);
        if display_back {
            let mut buf = [0u8;4];
            log(char::encode_utf8(res, &mut buf).as_bytes());
        }
        res
    }
}

pub const GUI_EVENT_EXIT: u16 = 0x00;
pub const GUI_EVENT_MOUSE_CLICK: u16 = 0x02;
pub const GUI_EVENT_TIME_UPDATE: u16 = 0x03;
pub const GUI_EVENT_UNDK_KEY: u16 = 0x04;

pub fn gui_event_make_ret(code: u16, arg0: u16, arg1: u16, arg2: u16) -> usize {
    ((code as usize) << 48) + ((arg2 as usize) << 32) + ((arg1 as usize) << 16) + (arg0 as usize)
}

pub fn gui_event_solve_ret(ret: usize) -> (u16, u16, u16, u16) {
    ((ret >> 48) as u16, ret as u16, (ret >> 16) as u16, (ret >> 32) as u16)
}

pub fn gui_sleep(million_seconds: usize) {
    unsafe { syscall!(REGISTER_TIMER, million_seconds) };
}

pub fn wait_gui_event() -> (u16, u16, u16, u16) {
    unsafe {
        let res = event_call!(GUI_PROGRAM);
        gui_event_solve_ret(res)
    }
}

pub fn register_time_update() {
    unsafe { syscall!(GUI_SUBSCRIBE_TIME_UPDATE) }; // 注册时间更新事件
}

pub fn register_gui_keyboard() {
    unsafe { syscall!(GUI_SUBSCRIBE_KEYBOARD) }; // 注册时间更新事件
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