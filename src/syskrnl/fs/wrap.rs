//! 本文件提供对fatfs的封装

use fatfs::Read;
use lazy_static::lazy_static;
use spin::Mutex;
use super::ata::AtaDeviceReader;
use super::time::CosTimeProvider;
use super::oem::Cp437Converter;

lazy_static! {
    static ref DATA_DISK_FS: Mutex<fatfs::FileSystem<AtaDeviceReader, CosTimeProvider, Cp437Converter>> = {
        let reader = AtaDeviceReader::new(0,1).unwrap();
        let option = fatfs::FsOptions::new().oem_cp_converter(Cp437Converter).time_provider(CosTimeProvider);
        let fs = fatfs::FileSystem::new(reader,option);
        Mutex::new(fs.unwrap())
    };
}

#[allow(dead_code)]
fn test() {
    let mut buf = [0u8; 100];
    let mut reader = AtaDeviceReader::new(0, 1).unwrap();
    reader.read(&mut buf).unwrap();

    println!("TEST ATA and ATA_READER:");
    for n in buf { print!("{:02X} ", n) }
    println!();

    println!("DATAFS Type: {:?}", DATA_DISK_FS.lock().fat_type());
}

pub fn seekpath(path: &str) {
    let fs_lock = DATA_DISK_FS.lock();
    let mut dir = fs_lock.root_dir();

}

pub fn metadata(path: &str) {
    unimplemented!()
}