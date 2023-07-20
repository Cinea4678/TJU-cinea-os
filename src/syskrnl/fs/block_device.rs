//! 该模块实现了块设备的抽象，包括内存块设备和ATA块设备。
//!
//! # 用法示例
//!
//! ```rust
//! use crate::block_device::{BlockDevice, MemBlockDevice};
//!
//! let mut dev = BlockDevice::Mem(MemBlockDevice::new(512, 1024));
//! let mut buf = [0u8; 512];
//!
//! // 从设备中读取数据
//! dev.read(0, &mut buf);
//!
//! // 向设备中写入数据
//! dev.write(0, &buf);
//!
//! // 获取设备块大小
//! let block_size = dev.block_size();
//! ```
//!
//! # 注意事项
//!
//! - 该模块提供了用于读取和写入块设备的函数，但需要注意设备地址和缓冲区大小的正确性。
//! - 该模块还包括用于管理文件系统中块的位图和超级块的结构体定义。
//! - 该模块提供了用于计算块索引和缓冲区索引的函数。
//! - 该模块定义了一个名为BlockDevice的枚举类型，包括内存块设备和ATA块设备。
//! - 该模块定义了一个名为BLOCK_DEVICE的全局变量，用于存储当前块设备的实例。

use alloc::vec;
use alloc::vec::Vec;

use spin::Mutex;

use crate::syskrnl;

use super::bitmap::BitmapBlock;
use super::dir::Dir;
use super::super_block::SuperBlock;

pub static BLOCK_DEVICE: Mutex<Option<BlockDevice>> = Mutex::new(None);

/// 块设备（枚举类型）
pub enum BlockDevice {
    Mem(MemBlockDevice),
    Ata(AtaBlockDevice),
}

/// 块设备公有操作
pub trait BlockDeviceIO {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), ()>;
    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), ()>;
    fn block_size(&self) -> usize;
    fn block_count(&self) -> usize;
}

impl BlockDeviceIO for BlockDevice {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), ()> {
        match self {
            BlockDevice::Mem(dev) => dev.read(addr, buf),
            BlockDevice::Ata(dev) => dev.read(addr, buf),
        }
    }

    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), ()> {
        match self {
            BlockDevice::Mem(dev) => dev.write(addr, buf),
            BlockDevice::Ata(dev) => dev.write(addr, buf),
        }
    }

    fn block_size(&self) -> usize {
        match self {
            BlockDevice::Mem(dev) => dev.block_size(),
            BlockDevice::Ata(dev) => dev.block_size(),
        }
    }

    fn block_count(&self) -> usize {
        match self {
            BlockDevice::Mem(dev) => dev.block_count(),
            BlockDevice::Ata(dev) => dev.block_count(),
        }
    }
}

pub fn is_mounted() -> bool {
    BLOCK_DEVICE.lock().is_some()
}

pub fn dismount() {
    *BLOCK_DEVICE.lock() = None;
}

/// 内存块设备
pub struct MemBlockDevice {
    dev: Vec<[u8; super::BLOCK_SIZE]>,
}

impl MemBlockDevice {
    pub fn new(len: usize) -> Self {
        let dev = vec![[0; super::BLOCK_SIZE]; len];
        Self { dev }
    }
}

impl BlockDeviceIO for MemBlockDevice {
    fn read(&mut self, block_index: u32, buf: &mut [u8]) -> Result<(), ()> {
        // TODO: check for overflow
        buf[..].clone_from_slice(&self.dev[block_index as usize][..]);
        Ok(())
    }

    fn write(&mut self, block_index: u32, buf: &[u8]) -> Result<(), ()> {
        // TODO: check for overflow
        self.dev[block_index as usize][..].clone_from_slice(buf);
        Ok(())
    }

    fn block_size(&self) -> usize {
        super::BLOCK_SIZE
    }

    fn block_count(&self) -> usize {
        self.dev.len()
    }
}

/// 把内存挂载为块设备
pub fn mount_mem() {
    let mem = syskrnl::allocator::avaliable_memory_size() / 2; // Half the allocatable memory
    let len = mem / super::BLOCK_SIZE; // TODO: take a size argument
    let dev = MemBlockDevice::new(len);
    *BLOCK_DEVICE.lock() = Some(BlockDevice::Mem(dev));
}

/// 将内存格式化
pub fn format_mem() {
    debug_assert!(is_mounted());
    if let Some(sb) = SuperBlock::new() {
        sb.write();
        let root = Dir::root();
        BitmapBlock::alloc(root.addr());
    }
}

const ATA_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct AtaBlockDevice {
    cache: [Option<(u32, Vec<u8>)>; ATA_CACHE_SIZE],
    dev: syskrnl::io::ata::Drive
}

impl AtaBlockDevice {
    /// 创建一个新的ATA设备对象
    pub fn new(bus: u8, dsk: u8) -> Option<Self> {
        syskrnl::io::ata::Drive::open(bus, dsk).map(|dev| {
            let cache = [(); ATA_CACHE_SIZE].map(|_| None);
            Self { dev, cache }
        })
    }

    /*
    pub fn len(&self) -> usize {
        self.block_size() * self.block_count()
    }
    */

    /// 将指定的数据块地址映射到缓存数组中的索引位置
    fn hash(&self, block_addr: u32) -> usize {
        (block_addr as usize) % self.cache.len()
    }

    /// 从缓存数组中获取指定的数据块，并返回一个 Option<&[u8]> 类型的值，表示获取到的数据块
    fn cached_block(&self, block_addr: u32) -> Option<&[u8]> {
        let h = self.hash(block_addr);
        if let Some((cached_addr, cached_buf)) = &self.cache[h] {
            if block_addr == *cached_addr {
                return Some(cached_buf);
            }
        }
        None
    }

    /// 设置缓存到的数据块
    fn set_cached_block(&mut self, block_addr: u32, buf: &[u8]) {
        let h = self.hash(block_addr);
        self.cache[h] = Some((block_addr, buf.to_vec()));
    }

    /// 取消缓存的数据块
    fn unset_cached_block(&mut self, block_addr: u32) {
        let h = self.hash(block_addr);
        self.cache[h] = None;
    }
}

impl BlockDeviceIO for AtaBlockDevice {
    /// 从 ATA 块设备中读取指定的数据块，并将数据块存储到指定的缓冲区中
    fn read(&mut self, block_addr: u32, buf: &mut [u8]) -> Result<(), ()> {
        if let Some(cached) = self.cached_block(block_addr) {
            buf.copy_from_slice(cached);
            return Ok(());
        }

        syskrnl::io::ata::read(self.dev.bus, self.dev.dsk, block_addr, buf)?;
        self.set_cached_block(block_addr, buf);
        Ok(())
    }

    /// 将指定的数据块写入到 ATA 块设备中
    fn write(&mut self, block_addr: u32, buf: &[u8]) -> Result<(), ()> {
        syskrnl::io::ata::write(self.dev.bus, self.dev.dsk, block_addr, buf)?;
        self.unset_cached_block(block_addr);
        Ok(())
    }

    fn block_size(&self) -> usize {
        self.dev.block_size() as usize
    }

    fn block_count(&self) -> usize {
        self.dev.block_count() as usize
    }
}

/// 将指定的 ATA 块设备挂载到文件系统中
pub fn mount_ata(bus: u8, dsk: u8) {
    *BLOCK_DEVICE.lock() = AtaBlockDevice::new(bus, dsk).map(BlockDevice::Ata);
}

/// 格式化 ATA 块设备
pub fn format_ata() {
    if let Some(sb) = SuperBlock::new() {
        // Write super_block
        sb.write();

        // Write zeros into block bitmaps
        super::bitmap::free_all();

        // Allocate root dir
        debug_assert!(is_mounted());
        let root = Dir::root();
        BitmapBlock::alloc(root.addr());
    }
}