//! 本文件提供对fatfs的封装

// 封装fatfs第一步：提供ata的RWS抽象

use core::cmp::min;

use fatfs::{IoBase, IoError, Read, Seek, SeekFrom, Write};

use ata::{BLOCK_BIN_SZ, BLOCK_MASK, BLOCK_SIZE};

use crate::syskrnl::io::ata;

pub struct AtaDeviceReader {
    pub bus: u8,
    pub device: u8,
    pub position: usize,
}

fn block(position: usize) -> u32 {
    (position >> BLOCK_BIN_SZ) as u32
}

fn offset(position: usize) -> usize {
    position & BLOCK_MASK
}

impl Read for AtaDeviceReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut block_buf = [0u8; BLOCK_SIZE];
        // 第一阶段：读取本块未完的部分
        ata::read(self.bus, self.device, block(self.position), &mut block_buf).expect("ATA read failed");
        let block_offset = offset(self.position);
        let next_read = BLOCK_SIZE - block_offset;
        let next_read = min(next_read, buf.len());
        buf[0..next_read].copy_from_slice(&block_buf[block_offset..block_offset + next_read]);
        let mut buf_pos = next_read;
        // 第二阶段：读取接下来的所有块


        Ok(0)
    }
}

impl IoBase for AtaDeviceReader { type Error = (); }

impl Write for AtaDeviceReader {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Seek for AtaDeviceReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        todo!()
    }
}

