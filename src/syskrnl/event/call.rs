use cinea_os_sysapi::event::*;
use x86_64::instructions::interrupts;

use crate::syskrnl;

use super::service;

pub fn dispatcher(event_id: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    interrupts::without_interrupts(|| {
        match event_id {
            KEYBOARD_INPUT => { unimplemented!() }
            SLEEP_WAKEUP => { service::sleep_wakeup(arg1) }
            _ => syskrnl::proc::id()
        }
    })
}