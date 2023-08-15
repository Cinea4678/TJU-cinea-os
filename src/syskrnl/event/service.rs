use cinea_os_sysapi::event::KEYBOARD_INPUT;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::syskrnl;
use crate::syskrnl::event::EVENT_QUEUE;
use crate::syskrnl::proc;

//
// EID段使用情况：
// 0..1_000_000 - 裸EID
// 1_000_000..2_000_000 - Sleep
// 2_000_000..3_000_000 - GUI
//

const SLEEP_EID_START: usize = 1_000_000;
pub const GUI_EID_START: usize = 2_000_000;

pub fn keyboard_input() -> usize {
    EVENT_QUEUE.lock().wait_for(KEYBOARD_INPUT)
}

static SLEEP_ID: AtomicUsize = AtomicUsize::new(SLEEP_EID_START);

pub fn sleep_wakeup(time: usize, register_only: bool) -> usize {
    let eid = SLEEP_ID.fetch_add(1, Ordering::SeqCst);
    if SLEEP_ID.load(Ordering::SeqCst) >= SLEEP_EID_START + 1_000_000 {
        SLEEP_ID.store(SLEEP_EID_START, Ordering::SeqCst);
    }
    syskrnl::time::add_sleep(time, eid);
    if register_only {
        // 指定不要停止调度
        EVENT_QUEUE.lock().wait_for_register_only(eid);
        0
    } else {
        let next = EVENT_QUEUE.lock().wait_for(eid);
        next
    }
}

pub fn gui_wakeup() -> usize {
    let eid = GUI_EID_START + proc::id();
    let next = EVENT_QUEUE.lock().wait_for(eid);
    next
}
