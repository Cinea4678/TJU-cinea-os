use alloc::boxed::Box;
use alloc::string::String;

use crate::syskrnl;

use super::{dirname, filename, realpath};
use super::block::LinkedBlock;
use super::dir_entry::DirEntry;
use super::FileType;
use super::super_block::SuperBlock;

#[derive(Debug, Clone)]
pub struct Dir {
    parent: Option<Box<Dir>>,
    name: String,
    addr: u32,
    size: u32,
    entry_index: u32,
}

impl From<DirEntry> for Dir {
    fn from(entry: DirEntry) -> Self {
        Self {
            parent: Some(Box::new(entry.dir())),
            name: entry.name(),
            addr: entry.addr(),
            size: entry.size(),
            entry_index: 0,
        }
    }
}

impl Dir {
    pub fn root() -> Self {
        let name = String::new();
        let addr = SuperBlock::read().data_area();
        let mut root = Self { parent: None, name, addr, size: 0, entry_index: 0 };
        root.update_size();
        root
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// 在指定的路径中创建一个新的目录
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname, syskrnl::proc::dir().as_str());
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_dir(filename) {
                return Some(dir_entry.into());
            }
        }
        None
    }

    /// 打开指令路径的目录
    pub fn open(pathname: &str) -> Option<Self> {
        if !super::is_mounted() {
            return None;
        }

        let mut dir = Dir::root();
        let pathname = realpath(pathname, syskrnl::proc::dir().as_str());

        if pathname == "/" {
            return Some(dir);
        }

        for name in pathname.trim_start_matches('/').split('/') {
            match dir.find(name) {
                Some(dir_entry) => {
                    if dir_entry.is_dir() {
                        dir = dir_entry.into()
                    } else {
                        return None;
                    }
                }
                None => {
                    return None;
                }
            }
        }
        Some(dir)
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    /// 在目录中查找指定名称的目录项
    pub fn find(&self, name: &str) -> Option<DirEntry> {
        self.entries().find(|entry| entry.name() == name)
    }

    /// 创建文件
    pub fn create_file(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::File, name)
    }

    /// 创建目录
    pub fn create_dir(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Dir, name)
    }

    /// 创建设备
    pub fn create_device(&mut self, name: &str) -> Option<DirEntry> {
        self.create_entry(FileType::Device, name)
    }

    /// 创建目录项
    fn create_entry(&mut self, kind: FileType, name: &str) -> Option<DirEntry> {
        if self.find(name).is_some() {
            return None;
        }

        // 遍历到目录的尾部
        let mut entries = self.entries();
        while entries.next().is_some() {}

        // 如果剩余空间不足了，就alloc一个新的块
        let space_left = entries.block.data().len() - entries.block_offset();
        let entry_len = DirEntry::empty_len() + name.len();
        if entry_len > space_left {
            match entries.block.alloc_next() {
                None => return None, // 磁盘满
                Some(new_block) => {
                    entries.block = new_block;
                    entries.block_offset = 0;
                }
            }
        }

        // 创建新的条目
        let entry_block = LinkedBlock::alloc().unwrap();
        let entry_kind = kind as u8;
        let entry_addr = entry_block.addr();
        let entry_size = 0u32;
        let entry_time = syskrnl::clock::realtime() as u64;
        let entry_name = truncate(name, u8::MAX as usize);
        let n = entry_name.len();
        let i = entries.block_offset();
        let data = entries.block.data_mut();

        data[i] = entry_kind;
        data[(i + 1)..(i + 5)].clone_from_slice(&entry_addr.to_be_bytes());
        data[(i + 5)..(i + 9)].clone_from_slice(&entry_size.to_be_bytes());
        data[(i + 9)..(i + 17)].clone_from_slice(&entry_time.to_be_bytes());
        data[i + 17] = n as u8;
        data[(i + 18)..(i + 18 + n)].clone_from_slice(entry_name.as_bytes());

        entries.block.write();
        self.update_size();

        Some(DirEntry::new(self.clone(), kind, entry_addr, entry_size, entry_time, &entry_name))
    }
}