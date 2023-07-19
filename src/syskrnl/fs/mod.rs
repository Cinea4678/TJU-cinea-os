pub mod dir;
pub mod dir_entry;

use crate::syskrnl;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum OpenFlag {
    Read = 1,
    Write = 2,
    Append = 4,
    Create = 8,
    Truncate = 16,
    Dir = 32,
    Device = 64
}

impl OpenFlag{
    /// 某个标志是否被设置
    fn is_set(&self, flags: usize) -> bool {
        flags & (*self as usize) != 0
    }
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Dir = 0,
    File = 1,
    Device = 2,
}

pub fn open(path: &str, flags: usize) -> Option<()> {
    unimplemented!()
}