/// 进程退出代码
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success        = 0,
    Failure        = 1,
    UsageError     = 64,
    DataError      = 65,
    OpenError      = 128,
    ReadError      = 129,
    ExecError      = 130,
    PageFaultError = 200,
    ShellExit      = 255,
}