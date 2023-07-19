use alloc::string::String;
use crate::syskrnl::fs::FileType;
use super::dir::Dir;

pub struct DirEntry {
    dir: Dir,
    addr: u32,

    // 文件信息
    kind: FileType,
    size: u32,
    time: u64,
    name: String
}

// impl DirEntry {
//     pub fn open(pathname: &str) -> Option<Self> {
//     }
// }
