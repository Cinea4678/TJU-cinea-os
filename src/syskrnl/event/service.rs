use core::sync::atomic::{AtomicUsize, Ordering};
use cinea_os_sysapi::event::KEYBOARD_INPUT;


use crate::syskrnl;
use crate::syskrnl::event::EVENT_QUEUE;

pub fn keyboard_input() -> usize {
    EVENT_QUEUE.lock().wait_for(KEYBOARD_INPUT)
}

static SLEEP_ID: AtomicUsize = AtomicUsize::new(1_000_000);

pub fn sleep_wakeup(time: usize) -> usize {
    let eid = SLEEP_ID.fetch_add(1, Ordering::SeqCst);
    if SLEEP_ID.load(Ordering::SeqCst) >= 2_000_000 {
        SLEEP_ID.store(1_000_000, Ordering::SeqCst);
    }
    let next = EVENT_QUEUE.lock().wait_for(eid);
    syskrnl::time::add_sleep(time, eid);
    next
}