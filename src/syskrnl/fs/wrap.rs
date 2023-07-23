//! 本文件提供对fatfs的封装

use alloc::string::String;
use alloc::vec::Vec;
use fatfs::{DirEntry, Read};
use lazy_static::lazy_static;
use spin::Mutex;

use cinea_os_sysapi::fs::{dirname, filename, Metadata, realpath};
use cinea_os_sysapi::fs as fsapi;
use cinea_os_sysapi::fs::FileError::NotADirError;

use fsapi::FileError::{self, NotFoundError, RootDirError};
use crate::syskrnl::proc;

use super::ata::AtaDeviceReader;
use super::oem::Cp437Converter;
use super::time::CosTimeProvider;


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

fn seekpath<'a, IO, TP, OCC>(path: &str, root_dir: fatfs::Dir<'a, IO, TP, OCC>)
                             -> Result<DirEntry<'a, IO, TP, OCC>, FileError>
    where IO: fatfs::ReadWriteSeek, TP: fatfs::TimeProvider, OCC: fatfs::OemCpConverter {
    // Split the path
    let path = realpath(path, proc::dir().as_str());
    let dirname = dirname(path.as_str());
    let filename = filename(path.as_str());
    let mut spilted_path: Vec<_> = dirname.split('/').filter(|x| { x.len() > 0 }).collect();
    fsapi::process_relative_path(&mut spilted_path)?;

    let mut dir = root_dir;

    for next in spilted_path {
        if let Ok(next_dir) = dir.open_dir(next) {
            dir = next_dir;
        } else {
            return Err(NotFoundError);
        }
    }

    if filename.len() == 0 {
        return Err(RootDirError);
    }

    if let Some(target) = dir.iter().find(|x| {
        if let Ok(x) = x {
            x.file_name() == filename
        } else {
            false
        }
    }) {
        Ok(target.unwrap())
    } else {
        Err(NotFoundError)
    }
}

/// 获取路径元数据
pub fn metadata(path: &str) -> Result<Metadata, FileError> {
    let lock = DATA_DISK_FS.lock();
    let entry = seekpath(path, lock.root_dir())?;
    Ok(fsapi::Metadata::from_dir_entry(entry))
}

/// 列出目录下的文件




/// 获取当前工作路径
pub fn current_dir() -> String {
    proc::dir().clone()
}

/// 更换路径
pub fn change_dir(path: &str) -> Result<(), FileError> {
    let meta = metadata(path)?;
    if !meta.is_dir() {
        Err(NotADirError)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use super::fsapi::process_relative_path;

    #[test_case]
    fn test_process_relative_path() {
        let mut test_set1 = vec!["foo", "bar"];
        let mut test_set2 = vec!["foo", ".", "bar"];
        let mut test_set3 = vec!["foo", ".", "bar", "."];
        let mut test_set4 = vec!["foo", "..", "foo", "bar"];
        let mut test_set5 = vec!["foo", ".", "..", "foo", "bar"];
        let mut test_set6 = vec!["foo", "bar", "."];
        let mut test_set7 = vec!["."];
        let mut test_set8 = vec![".."];
        let mut test_set9 = vec![".", ".."];
        let mut test_set10 = vec!["foo", "bar", "foo", "bar", "..", "..", ".."];
        let mut test_set11 = vec!["foo", "bar", "foo", "bar", "..", "..", "..", ".."];
        let mut test_set12 = vec!["foo", "bar", "foo", "bar", "..", "..", "..", "..", ".."];

        process_relative_path(&mut test_set1).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set1);
        process_relative_path(&mut test_set2).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set2);
        process_relative_path(&mut test_set3).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set3);
        process_relative_path(&mut test_set4).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set4);
        process_relative_path(&mut test_set5).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set5);
        process_relative_path(&mut test_set6).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set6);
        process_relative_path(&mut test_set7).unwrap();
        assert_eq!(vec!["foo", "bar"], test_set6);
        process_relative_path(&mut test_set8).unwrap_err();
        process_relative_path(&mut test_set9).unwrap_err();
        process_relative_path(&mut test_set10).unwrap();
        assert_eq!(vec!["foo"], test_set10);
        process_relative_path(&mut test_set11).unwrap();
        assert_eq!(alloc::vec::Vec::<&str>::new(), test_set11);
        process_relative_path(&mut test_set12).unwrap_err();
        println!("[ok]  FileSystem API test_process_relative_path")
    }

    #[test_case]
    fn test_realpath() {
        use super::fsapi::realpath;
        assert_eq!(realpath("/usr/bin/gcc", "/home/user"), "/usr/bin/gcc");
        assert_eq!(realpath("gcc", "/usr/bin"), "/usr/bin/gcc");
        assert_eq!(realpath("gcc", "/usr/bin/"), "/usr/bin/gcc");
        println!("[ok]  FileSystem API test_realpath")
    }

    #[test_case]
    fn test_filename() {
        use super::fsapi::filename;
        assert_eq!(filename("/usr/bin/gcc"), "gcc");
        assert_eq!(filename("/usr/bin/"), "");
        assert_eq!(filename("/usr/"), "");
        assert_eq!(filename("/"), "");
        assert_eq!(filename("gcc"), "gcc");
        println!("[ok]  FileSystem API test_filename")
    }

    #[test_case]
    fn test_dirname() {
        use super::fsapi::dirname;
        assert_eq!(dirname("/usr/bin/gcc"), "/usr/bin");
        assert_eq!(dirname("/usr/bin/"), "/usr/bin");
        assert_eq!(dirname("/usr/"), "/usr");
        assert_eq!(dirname("/"), "/");
        assert_eq!(dirname("gcc"), "gcc");
        println!("[ok]  FileSystem API test_dirname")
    }
}
