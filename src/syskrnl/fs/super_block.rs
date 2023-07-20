use crate::syskrnl;
use crate::syskrnl::fs::block_device::BlockDeviceIO;
use super::block::Block;

const SUPERBLOCK_ADDR: u32 = (crate::KERNEL_SIZE / super::BLOCK_SIZE) as u32;
const SIGNATURE: &[u8; 8] = b"CINEA FS";

#[derive(Debug)]
pub struct SuperBlock {
    signature: &'static[u8; 8],
    version: u8,
    block_size: u32,
    pub block_count: u32,
    pub alloc_count: u32,
}

impl SuperBlock {
    /// 检查ATA设备是否为本文件系统
    pub fn check_ata(bus: u8, dsk: u8) -> bool {
        let mut buf = [0u8; super::BLOCK_SIZE];
        if syskrnl::io::ata::read(bus, dsk, SUPERBLOCK_ADDR, &mut buf).is_err() {
            return false;
        }
        &buf[0..8] == SIGNATURE
    }

    /// 新的超级块
    pub fn new() -> Option<Self> {
        if let Some(ref dev) = *super::block_device::BLOCK_DEVICE.lock() {
            Some(Self {
                signature: SIGNATURE,
                version: 1,
                block_size: dev.block_size() as u32,
                block_count: dev.block_count() as u32,
                alloc_count: 0,
            })
        } else {
            None
        }
    }

    /// 读取超级块
    pub fn read() -> Self {
        let block = Block::read(SUPERBLOCK_ADDR);
        let data = block.data();
        debug_assert_eq!(&data[0..8], SIGNATURE);
        Self {
            signature: SIGNATURE,
            version: data[8],
            block_size: 2 << (8 + data[9] as u32),
            block_count: u32::from_be_bytes(data[10..14].try_into().unwrap()),
            alloc_count: u32::from_be_bytes(data[14..18].try_into().unwrap()),
        }
    }

    /// 写入超级块
    pub fn write(&self) {
        let mut block = Block::new(SUPERBLOCK_ADDR);
        let data = block.data_mut();

        data[0..8].clone_from_slice(self.signature);
        data[8] = self.version;

        let size = self.block_size;
        debug_assert!(size >= 512);
        debug_assert!(size.is_power_of_two());
        data[9] = (size.trailing_zeros() as u8) - 9; // 2 ^ (9 + n)
        data[10..14].clone_from_slice(&self.block_count.to_be_bytes());
        data[14..18].clone_from_slice(&self.alloc_count.to_be_bytes());

        block.write();
    }

    /// 超级块的大小
    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    /// 超级块的块数量
    pub fn block_count(&self) -> u32 {
        self.block_count
    }

    /// 返回位图区域的起始地址
    pub fn bitmap_area(&self) -> u32 {
        SUPERBLOCK_ADDR + 2
    }

    /// 返回数据区域的起始地址
    pub fn data_area(&self) -> u32 {
        let bs = super::BITMAP_SIZE as u32;
        let total = self.block_count;
        let offset = self.bitmap_area();
        let rest = (total - offset) * bs / (bs + 1);
        self.bitmap_area() + rest / bs
    }
}

/// 增加分配块数量
pub fn inc_alloc_count() {
    let mut sb = SuperBlock::read();
    sb.alloc_count += 1;
    sb.write();
}

/// 减少分配块数量
pub fn dec_alloc_count() {
    let mut sb = SuperBlock::read();
    sb.alloc_count -= 1;
    sb.write();
}
