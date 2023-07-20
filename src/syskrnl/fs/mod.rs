//!
//! # 文件系统
//!
//! 本部分参考Moros完成。毕竟，有前人的优秀轮子在前，比起自己嗯造一个，学习和借鉴才是最明智的选择。
//!
//! 再加上，现在本来也就没有多少时间了（悲）
//!

pub mod dir;
pub mod dir_entry;
pub mod super_block;
pub mod block;
pub mod bitmap;
pub mod block_device;

use crate::syskrnl;

pub use bitmap::BITMAP_SIZE;
pub use block_device::is_mounted;
pub use cinea_os_sysapi::fs::{dirname, filename, realpath, FileIO};
pub use crate::syskrnl::io::ata::BLOCK_SIZE;

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