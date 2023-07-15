pub const EXIT:     usize = 0x1;
pub const SPAWN:    usize = 0x2;
pub const READ:     usize = 0x3;
pub const WRITE:    usize = 0x4;
pub const OPEN:     usize = 0x5;
pub const CLOSE:    usize = 0x6;
pub const INFO:     usize = 0x7;
pub const DUP:      usize = 0x8;
pub const DELETE:   usize = 0x9;
pub const STOP:     usize = 0xA;
pub const SLEEP:    usize = 0xB;
/// 打印日志 (2): a0-msg, a1-len
pub const LOG:      usize = 0xC;
/// 分配堆内存 (2): a0-size a1-align ret-ptr(usize)
pub const ALLOC:    usize = 0xD;
/// 归还堆内存 (3): a0-ptr a1-size a2-align
pub const FREE:     usize = 0xE;
pub const PANIC:    usize = 0xF;