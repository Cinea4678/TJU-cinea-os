//! 处理进程的Sleep

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;

use conquer_once::spin::OnceCell;
use crossbeam::queue::ArrayQueue;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::syskrnl::event::{EVENT_QUEUE, EventType};
use crate::syskrnl::time;
use crate::syskrnl::time::TICKS_PER_SECOND;

lazy_static! {
    pub static ref WAKEUP_QUEUE: Mutex<VecDeque<usize>> = {
        Mutex::new(VecDeque::new())
    };
    pub static ref WAKEUP_MAP: Mutex<BTreeMap<usize, VecDeque<EventType>>> = {
        Mutex::new(BTreeMap::new())
    };
}


pub fn init() {}

pub fn check_wakeup() -> Option<usize> {
    let now = time::ticks();
    let mut queue = WAKEUP_QUEUE.lock();
    if let Some(first) = queue.get(0).cloned() {
        if first <= now {
            queue.pop_front();
            let mut lock = WAKEUP_MAP.lock();
            let map_queue = lock.get_mut(&first).unwrap();
            if let Some(eid) = map_queue.pop_front() {
                if map_queue.len() == 0 {
                    lock.remove(&first);
                }
                EVENT_QUEUE.lock().wakeup(eid)
            } else { None }
        } else { None }
    } else { None }
}

pub fn add_sleep(time: usize, eid: usize) {
    let now = time::ticks();
    let wakeup_time = now + time * TICKS_PER_SECOND / 1000;
    WAKEUP_QUEUE.lock().push_back(wakeup_time);
    WAKEUP_MAP.lock().entry(wakeup_time).or_insert(VecDeque::new()).push_back(eid);
}
