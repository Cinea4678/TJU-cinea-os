use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;

pub use call::dispatcher;

use crate::syskrnl;
use crate::syskrnl::proc::SCHEDULER;

pub mod call;
mod service;

pub static NEED_CHECK_EVENT_DATA: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref EVENT_DATA:Mutex<BTreeMap<usize,usize>> = {
        Mutex::new(BTreeMap::new())
    };

    pub static ref EVENT_QUEUE:Mutex<EventQueue> = {
        Mutex::new(EventQueue::new())
    };
}

pub type EventType = usize;

pub struct EventQueue {
    /// 队列本体
    queue: BTreeMap<EventType, VecDeque<usize>>,
    /// 前台进程
    front_proc: usize,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            front_proc: 1,
        }
    }

    /// 切换前台进程
    pub fn switch_front(&mut self, new_front: usize) {
        self.front_proc = new_front
    }

    /// 注册事件等待
    fn wait(&mut self, pid: usize, eid: usize) {
        self.queue.entry(eid).or_insert(VecDeque::new()).push_back(pid)
    }

    /// 等待某事件
    ///
    /// 返回值是下一个进程的pid
    pub fn wait_for(&mut self, event: EventType) -> usize {
        // 标识当前进程为“等待”，停止其调度
        let next = SCHEDULER.lock().wait();
        // 注册事件等待
        self.wait(syskrnl::proc::id(), event);
        // 返回下一个进程的PID
        return next;
    }

    /// 根据事件唤醒进程
    ///
    /// 返回值是下一个进程的pid
    pub fn wakeup(&mut self, event: EventType) -> Option<usize> {
        if let Some(mut queue) = self.queue.get_mut(&event) {
            if queue.len() == 0 { None } else {
                if let Some(fp) = queue.iter().position(|x| *x == self.front_proc) {
                    queue.remove(fp).unwrap();
                    Some(self.front_proc)
                } else {
                    Some(queue.pop_front().unwrap())
                }
            }
        } else {
            None
        }
    }

    /// 根据事件唤醒进程，并且指定返回值
    pub fn wakeup_with_ret(&mut self, event: EventType, ret: usize) -> Option<usize> {
        if let Some(pid) = self.wakeup(event) {
            NEED_CHECK_EVENT_DATA.store(true, Ordering::Relaxed);
            let mut lock = EVENT_DATA.lock();
            *lock.entry(pid).or_insert(0) = ret;
            Some(pid)
        } else {
            None
        }
    }
}