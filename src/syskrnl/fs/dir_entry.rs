use alloc::string::String;
use alloc::vec::Vec;
use cinea_os_sysapi::fs::{dirname, filename, realpath};
use crate::syskrnl::fs::FileType;
use crate::syskrnl::proc;
use super::dir::Dir;

pub struct DirEntry {
    dir: Dir,
    addr: u32,

    // 文件信息
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}

impl DirEntry {
    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname, proc::dir().as_str());
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            return dir.find(filename);
        }
        None
    }

    pub fn new(dir: Dir, kind: FileType, addr: u32, size: u32, time: u64, name: &str) -> Self {
        let name = String::from(name);
        Self { dir, kind, addr, size, time, name }
    }

    pub fn empty_len() -> usize {
        1 + 4 + 4 + 8 + 1  // ？这个之后研究一下吧
    }

    pub fn len(&self) -> usize {
        Self::empty_len() + self.name.len()
    }

    pub fn is_empty(&self) -> bool {
        Self::empty_len() == self.len()
    }

    pub fn kind(&self) -> FileType {
        self.kind
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn is_device(&self) -> bool {
        self.kind == FileType::Device
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn dir(&self) -> Dir {
        self.dir.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    // pub fn info(&self) -> FileInfo {
    //     FileInfo { kind: self.kind, name: self.name(), size: self.size(), time: self.time }
    // }
}

#[derive(Debug)]
pub struct FileInfo {
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}

impl FileInfo {
    pub fn new() -> Self {
        Self { kind: FileType::File, name: String::new(), size: 0, time: 0 }
    }

    pub fn root() -> Self {
        let kind = FileType::Dir;
        let name = String::new();
        let size = Dir::root().size() as u32;
        let time = 0;
        Self { kind, name, size, time }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn kind(&self) -> FileType {
        self.kind
    }

    // TODO: Duplicated from dir entry
    pub fn is_dir(&self) -> bool {
        self.kind == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileType::File
    }

    pub fn is_device(&self) -> bool {
        self.kind == FileType::Device
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        debug_assert!(self.name.len() < 256);
        let mut res = Vec::new();
        res.push(self.kind as u8);
        res.extend_from_slice(&self.size.to_be_bytes());
        res.extend_from_slice(&self.time.to_be_bytes());
        res.push(self.name.len() as u8);
        res.extend_from_slice(self.name.as_bytes());
        res
    }
}
