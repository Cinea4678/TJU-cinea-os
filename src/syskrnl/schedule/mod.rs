pub mod roundroll;

use crate::syskrnl::proc::Process;

/// 进程调度器
pub trait ProcessScheduler: Send {
    /// 注册新进程
    /// 返回 - PID
    fn add(&mut self, process: Process, priority: u32) -> usize;

    /// 取消进程
    fn terminate(&mut self, process: Process) -> usize;

    /// 时间到轮换
    fn timeup(&mut self) -> usize;

    /// 主动退出运行
    fn giveup(&mut self)->usize;

    /// 进程等待
    fn wait(&mut self)->usize;

    /// 进程唤醒
    fn wakeup(&mut self, process:usize)->usize;
}

