use alloc::boxed::Box;
use alloc::vec::Vec;

use lazy_static::lazy_static;
use spin::mutex::Mutex;

use cinea_os_sysapi::fs::{FileError, FileIO};

lazy_static! {
    static ref DEVICE_TABLE: Mutex<Vec<Box::<dyn FileIO>>> = {
        let v: Vec<Box::<dyn FileIO>> = Vec::new();

        Mutex::new(v)
    };
}

pub fn read(id: usize, buf: &mut [u8]) -> Result<usize, FileError> {
    if id >= DEVICE_TABLE.lock().len() {
        Err(FileError::NotFoundError)
    } else {
        match DEVICE_TABLE.lock()[id].read(buf) {
            Ok(len) => Ok(len),
            Err(()) => Err(FileError::DeviceIOError)
        }
    }
}


pub fn write(id: usize, buf: &[u8]) -> Result<usize, FileError> {
    if id >= DEVICE_TABLE.lock().len() {
        Err(FileError::NotFoundError)
    } else {
        match DEVICE_TABLE.lock()[id].write(buf) {
            Ok(len) => Ok(len),
            Err(()) => Err(FileError::DeviceIOError)
        }
    }
}
