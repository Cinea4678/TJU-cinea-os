pub mod call;

pub use call::dispatcher;

use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, AtomicUsize};

use crate::syskrnl::proc::SCHEDULER;

pub static HAS_EVENT_DATA: AtomicBool = AtomicBool::new(false);
pub static EVENT_DATA: AtomicUsize = AtomicUsize::new(0);

pub type EventType = usize;

pub type EventTuple = (usize, EventType);

pub struct EventQueue {
    /// 队列本体
    queue: VecDeque<EventTuple>,
    /// 前台进程
    front_proc: usize,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            front_proc: 1,
        }
    }

    /// 切换前台进程
    pub fn switch_front(&mut self, new_front: usize) {
        self.front_proc = new_front
    }

    /// 注册事件等待
    fn wait(&mut self, event_tuple: EventTuple) {
        self.queue.push_back(event_tuple)
    }

    /// 等待某事件
    ///
    /// 返回值是下一个进程的pid
    pub fn wait_for(&mut self, event: EventType)->usize {
        // 标识当前进程为“等待”，停止其调度
        let next = SCHEDULER.lock().wait();
        //
    }
}