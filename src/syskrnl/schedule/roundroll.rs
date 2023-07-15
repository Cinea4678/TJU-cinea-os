use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::syskrnl::proc::Process;
use crate::syskrnl::schedule::ProcessScheduler;

struct RoundRollNode {
    /// pid
    pid: usize,

    /// 是否空闲
    empty: bool,

    /// 是否可跳过
    skip: bool,

    /// 指向下一任务的指针
    next: usize,

    /// 指向上一任务的指针
    prev: usize,
}

/// 轮转算法
pub struct RoundRollScheduler {
    /// 进程表
    table: Vec<RoundRollNode>,

    /// PID对应表
    map: BTreeMap<usize, usize>,

    /// 头指针
    head: usize,

    /// 空节点指针
    empty: usize,

    /// 当前节点
    cursor: usize,
}

impl RoundRollNode {
    pub fn new() -> Self {
        RoundRollNode {
            pid: 0,
            empty: false,
            skip: false,
            next: 0,
            prev: 0,
        }
    }
}

impl RoundRollScheduler {
    pub fn new() -> Self {
        let mut s = RoundRollScheduler {
            table: vec![RoundRollNode::new()],
            map: BTreeMap::new(),
            head: 0,
            empty: 0,
            cursor: 0,
        };
        s.map.insert(0, 0);  // 插入0-0映射
        s.table[0].skip = true;
        s
    }

    /// IN 调度一个新节点
    fn alloc(&mut self) -> usize {
        if self.table[self.empty].empty {
            let id = self.empty;
            self.empty = self.table[id].next;
            id
        } else {
            self.table.push(RoundRollNode::new());
            self.table.len() - 1
        }
    }

    /// IN 归还一个节点
    fn dealloc(&mut self, node: usize) {
        if self.head == node {
            self.head = self.table[node].next;
        }
        let (prev, next) = (self.table[node].prev, self.table[node].next);
        self.table[prev].next = next;
        self.table[next].prev = prev;
        self.table[node].empty = true;
        self.table[node].next = self.empty;
        self.empty = node;
    }

    /// 加入新进程
    pub fn add(&mut self, process_id: usize) {
        let node = self.alloc();
        self.table[node].pid = process_id;
        let prev = self.table[self.head].prev;
        self.table[prev].next = node;
        self.table[self.head].prev = node;
        self.map.insert(process_id, node);
    }

    /// 移除旧进程
    pub fn remove(&mut self, process_id: usize) {
        if let Some(node) = self.map.get(&process_id) {
            let node = *node;
            if self.cursor == node {
                self.step();
            }
            self.dealloc(node);
            self.map.remove(&process_id).unwrap();
        }
    }

    /// 获取当前PID
    pub fn now(&self) -> usize {
        self.table[self.cursor].pid
    }

    /// 向后进一步
    pub fn step(&mut self) -> usize {
        self.cursor = self.table[self.cursor].next;
        while self.table[self.cursor].skip {
            self.cursor = self.table[self.cursor].next;
        }
        self.now()
    }
}

impl ProcessScheduler for RoundRollScheduler {
    fn add(&mut self, process: Process, _priority: u32) -> usize {
        self.add(process.id);
        self.cursor = *self.map.get(&process.id).unwrap();
        self.now()
    }

    fn terminate(&mut self, process: Process) -> usize {
        self.remove(process.id);
        self.now()
    }

    fn timeup(&mut self) -> usize {
        self.step()
    }

    fn giveup(&mut self) -> usize {
        self.step()
    }

    fn wait(&mut self) -> usize {
        self.table[self.cursor].skip = true;
        self.step()
    }

    fn wakeup(&mut self, process: Process) -> usize {
        if let Some(node) = self.map.get(&process.id) {
            self.table[*node].skip = false;
        }
        self.now()
    }
}