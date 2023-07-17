use alloc::collections::VecDeque;

/// 唤醒器，多进程/任务里面所使用的
pub trait Waker{
    /// 立即唤醒某个进程
    fn wake(pid: usize);

    /// 在未来time秒后唤醒某个进程
    fn wake_at(pid:usize, time: usize);

    /// 获取此时所需要被唤醒的进程
    fn wake_list()->Option<usize>;
}

pub struct SimpleWaker{
    /// 进程列表，usize与f64分别指进程PID和唤醒时间
    process_list: VecDeque<(usize,f64)>
}