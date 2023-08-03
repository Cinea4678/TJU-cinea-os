use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;

use lazy_static::lazy_static;
use spin::Mutex;

use cinea_os_sysapi::fs::{FileError, FileIO};

lazy_static! {
    static ref DEVICE_TABLE: Mutex<BTreeMap<String, Box::<dyn FileIO>>> = {
        let mut m: BTreeMap<String, Box::<dyn FileIO>> = BTreeMap::new();

        m.insert(String::from("/dev/stdout"), Box::new(crate::syskrnl::io::StdOutDevice));
        m.insert(String::from("/dev/uptime"), Box::new(crate::syskrnl::time::UpTimeDevice));

        Mutex::new(m)
    };
}

pub fn is_device(path: &str) -> bool {
    DEVICE_TABLE.lock().contains_key(path)
}

pub fn read(path: &str, buf: &mut [u8]) -> Result<usize, FileError> {
    let mut lock = DEVICE_TABLE.lock();
    match lock.get_mut(path) {
        None => Err(FileError::NotFoundError),
        Some(device) => {
            match device.read(buf) {
                Ok(len) => Ok(len),
                Err(()) => Err(FileError::DeviceIOError)
            }
        }
    }
}

pub fn write(path: &str, buf: &[u8]) -> Result<usize, FileError> {
    let mut lock = DEVICE_TABLE.lock();
    match lock.get_mut(path) {
        None => Err(FileError::NotFoundError),
        Some(device) => {
            match device.write(buf) {
                Ok(len) => Ok(len),
                Err(()) => Err(FileError::DeviceIOError)
            }
        }
    }
}

// use byteorder::{ByteOrder, LittleEndian};
//
// const DEVICE_FILE_SIG: [u8; 5] = [b'C', b'E', b'D', b'V', b'C'];
//
// /// Read device id from file blocks!
// pub fn device_id(block: &[u8]) -> Result<usize, FileError> {
//     if block.len() < 9 { Err(FileError::NotADeviceError) } else if DEVICE_FILE_SIG != block[0..5] { Err(FileError::NotADeviceError) } else { Ok(LittleEndian::read_u64(&block[5..9]) as usize) }
// }
