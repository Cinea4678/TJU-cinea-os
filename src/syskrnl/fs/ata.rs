use core::cmp::min;

use fatfs::{IoBase, Read, Seek, SeekFrom, Write};

use ata::{BLOCK_BIN_SZ, BLOCK_MASK, BLOCK_SIZE};

use crate::syskrnl::io::ata;

pub struct AtaDeviceReader {
    pub bus: u8,
    pub device: u8,
    max_block: u32,
    position: usize,
    cache: [u8; BLOCK_SIZE],
    writing: bool,
}

fn block(position: usize) -> u32 {
    (position >> BLOCK_BIN_SZ) as u32
}

fn offset(position: usize) -> usize {
    position & BLOCK_MASK
}

impl AtaDeviceReader {
    pub fn new(bus: u8, device: u8) -> Result<Self, ()> {
        if let Some(drive) = ata::Drive::open(bus, device) {
            let max_block = drive.block_count();
            Ok(Self {
                bus,
                device,
                max_block,
                position: 0,
                cache: [0u8; BLOCK_SIZE],
                writing: false,
            })
        } else {
            Err(())
        }
    }

    pub fn read_block(&mut self) -> Result<(), ()> {
        let block = block(self.position);
        if block < self.max_block {
            ata::read(self.bus, self.device, block, &mut self.cache)?;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_block(&mut self) -> Result<(), ()> {
        let block = block(self.position);
        if block < self.max_block {
            ata::write(self.bus, self.device, block, &self.cache)?;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_position(&mut self, pos: usize) -> Result<(), ()> {
        if pos > self.max_block as usize * BLOCK_SIZE { return Err(()) };
        self.position = pos;
        Ok(())
    }
}

impl Read for AtaDeviceReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.writing {
            self.flush().expect("Flush Fail");
            self.writing = false;
        }
        // 第一阶段：读取本块未完的部分
        if let Err(_) = self.read_block() { return Ok(0); }
        let block_offset = offset(self.position);
        let next_read = min(BLOCK_SIZE - block_offset, buf.len());
        buf[0..next_read].copy_from_slice(&self.cache[block_offset..block_offset + next_read]);
        let mut buf_pos = next_read;
        self.position += next_read;
        // 第二阶段：读取接下来的所有块
        while buf_pos < buf.len() {
            if let Err(_) = self.read_block() { return Ok(buf_pos); }
            let next_read = min(BLOCK_SIZE, buf.len() - buf_pos);
            buf[buf_pos..buf_pos + next_read].copy_from_slice(&self.cache[0..next_read]);
            self.position += next_read;
            buf_pos += next_read;
        }
        Ok(buf_pos)
    }
}

impl IoBase for AtaDeviceReader { type Error = (); }

impl Write for AtaDeviceReader {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if !self.writing {
            self.read_block().expect("flush Fail");
            self.writing = true;
        }
        // 第一阶段：写完本块未写完的部分
        let block_offset = offset(self.position);
        let next_write = min(BLOCK_SIZE - block_offset, buf.len());
        self.cache[block_offset..block_offset + next_write].copy_from_slice(&buf[0..next_write]);
        if let Err(_) = self.flush() { return Ok(0) };
        let mut buf_pos = next_write;
        self.position += next_write;
        // 第二阶段：写入接下来的所有块
        while buf_pos < buf.len() {
            let next_write = min(BLOCK_SIZE, buf.len() - buf_pos);
            self.cache[0..next_write].copy_from_slice(&buf[buf_pos..buf_pos + next_write]);
            if let Err(_) = self.flush() { return Ok(buf_pos) };
            self.position += next_write;
            buf_pos += next_write;
        }
        Ok(buf_pos)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.write_block()
    }
}

impl Seek for AtaDeviceReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Start(pos) => {
                self.set_position(pos as usize)?;
                Ok(pos)
            }
            SeekFrom::End(npos) => {
                self.set_position((self.max_block as i64 * BLOCK_SIZE as i64 - npos) as usize)?;
                Ok(self.position as u64)
            }
            SeekFrom::Current(rpos) => {
                self.set_position((self.position as i64 + rpos) as usize)?;
                Ok(self.position as u64)
            }
        }
    }
}

