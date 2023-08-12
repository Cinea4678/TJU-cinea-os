use alloc::vec;
use alloc::vec::Vec;

use fatfs::{IoBase, Read, Seek, SeekFrom, Write};

use crate::syskrnl::io;
use crate::syskrnl::io::ahci::HbaPort;

const SECTOR_SIZE: usize = 512;
const SECTOR_BIN_SZ: usize = 9;
const SECTOR_MASK: usize = 0x1FF;
const CACHE_SECTORS: u64 = 64;
const CACHE_SIZE: usize = SECTOR_SIZE * CACHE_SECTORS as usize;  // 32KB

pub struct AhciDeviceReader {
    pub port_no: u64,
    max_sector: u64,
    position: usize,
    cache: Vec<u8>,
    port: &'static mut HbaPort,
}

fn sector(position: usize) -> u64 {
    (position >> SECTOR_BIN_SZ) as u64
}

fn offset(position: usize) -> usize {
    position & SECTOR_MASK
}

impl AhciDeviceReader {
    pub fn new(port_no: u64) -> Result<Self, ()> {
        if let Some(port) = io::ahci::get_port(port_no as usize) {
            io::ahci::port_rebase(port, port_no);
            if let Some(max_sector) = port.get_max_sectors() {
                let cache = vec![0u8; CACHE_SIZE];
                Ok(AhciDeviceReader {
                    port_no,
                    max_sector,
                    position: 0,
                    cache,
                    port,
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn set_position(&mut self, pos: usize) -> Result<(), ()> {
        if pos > self.max_sector as usize * SECTOR_SIZE { return Err(()); };
        self.position = pos;
        Ok(())
    }
}

impl IoBase for AhciDeviceReader { type Error = (); }

impl Read for AhciDeviceReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let read_len = buf.len().min(self.max_sector as usize * SECTOR_SIZE - self.position);
        self.read_exact(&mut buf[0..read_len])?;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        if self.position + buf.len() > self.max_sector as usize * SECTOR_SIZE { return Err(()); };

        // 计算总共需要读取的扇区
        let sectors = (offset(self.position) + buf.len()) / SECTOR_SIZE + 1;
        let read_times = sectors / CACHE_SECTORS as usize;
        let start_sector = sector(self.position);

        let mut buf_pos = 0;
        let mut last_read = 0;

        for i in 0..=read_times {
            self.cache.fill(0);
            if i == read_times {
                // 最后一次
                if !self.port.read(start_sector + i as u64 * CACHE_SECTORS, (sectors % CACHE_SECTORS as usize) as u32, self.cache.as_mut_slice()) {
                    return Err(());
                }
                last_read = (sectors % CACHE_SECTORS as usize) * SECTOR_SIZE;
            } else {
                if !self.port.read(start_sector + i as u64 * CACHE_SECTORS, CACHE_SECTORS as u32, self.cache.as_mut_slice()) {
                    return Err(());
                }
                last_read = CACHE_SIZE;
            }
            if i == 0 {
                let next_copy = (last_read - offset(self.position)).min(buf.len() - buf_pos);
                buf.copy_from_slice(&self.cache.as_slice()[offset(self.position)..offset(self.position) + next_copy]);
                buf_pos += next_copy;
            } else {
                let next_copy = (last_read).min(buf.len() - buf_pos);
                &(buf[buf_pos..]).copy_from_slice(&self.cache.as_slice()[0..next_copy]);
                buf_pos += next_copy;
            }
        }

        self.position += buf.len();

        Ok(())
    }
}

impl Write for AhciDeviceReader{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let write_len = buf.len().min(self.max_sector as usize * SECTOR_SIZE - self.position);
        self.write_all(&buf[0..write_len])?;
        Ok(write_len)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        if self.position + buf.len() > self.max_sector as usize * SECTOR_SIZE { return Err(()); };

        // 计算总共需要写入的扇区
        let sectors = (offset(self.position) + buf.len()) / SECTOR_SIZE + 1;
        let write_times = sectors / CACHE_SECTORS as usize;
        let start_sector = sector(self.position);

        let mut buf_pos = 0;
        let mut last_write = 0;

        for i in 0..=write_times {
            self.cache.fill(0);
            if i == 0 {
                let next_copy = (last_write - offset(self.position)).min(buf.len() - buf_pos);
                self.cache.copy_from_slice(&buf[offset(self.position)..offset(self.position) + next_copy]);
                buf_pos += next_copy;
            } else {
                let next_copy = (last_write).min(buf.len() - buf_pos);
                &(self.cache.as_mut_slice()[buf_pos..]).copy_from_slice(&buf[0..next_copy]);
                buf_pos += next_copy;
            }
            if i == write_times {
                // 最后一次
                if !self.port.write(start_sector + i as u64 * CACHE_SECTORS, (sectors % CACHE_SECTORS as usize) as u32, self.cache.as_slice()) {
                    return Err(());
                }
                last_write = (sectors % CACHE_SECTORS as usize) * SECTOR_SIZE;
            } else {
                if !self.port.write(start_sector + i as u64 * CACHE_SECTORS, CACHE_SECTORS as u32, self.cache.as_slice()) {
                    return Err(());
                }
                last_write = CACHE_SIZE;
            }
        }

        self.position += buf.len();

        Ok(())

    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(()) // 直接不用缓冲区
    }
}

impl Seek for AhciDeviceReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos{
            SeekFrom::Start(pos) => {
                self.set_position(pos as usize)?;
                Ok(pos)
            }
            SeekFrom::End(npos) => {
                self.set_position((self.max_sector as i64 * SECTOR_SIZE as i64 - npos) as usize)?;
                Ok(self.position as u64)
            }
            SeekFrom::Current(rpos) => {
                self.set_position((self.position as i64 + rpos) as usize)?;
                Ok(self.position as u64)
            }
        }
    }
}

