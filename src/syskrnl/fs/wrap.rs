//! 本文件提供对fatfs的封装

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use core::sync::atomic::{AtomicUsize, Ordering};

use fatfs::{DirEntry, Read, Seek, SeekFrom, Write};
use lazy_static::lazy_static;
use spin::Mutex;

use cinea_os_sysapi::fs::{dirname, FileEntry, filename, Metadata, path_combine, realpath};
use cinea_os_sysapi::fs as fsapi;
use cinea_os_sysapi::fs::FileError::{NotAFileError, OSError};
use fsapi::FileError::{self, NotADirError, NotFoundError, RootDirError};

use crate::syskrnl::fs::device::is_device;
use crate::syskrnl::proc;
use crate::syskrnl::proc::{file_handles, set_dir};

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

// fn is_device<IO, TP, OCC>(file: &mut fatfs::File<IO, TP, OCC>) -> Option<usize>
//     where IO: fatfs::ReadWriteSeek, TP: fatfs::TimeProvider, OCC: fatfs::OemCpConverter {
//     let mut buf = [0u8; 9];
//     if let Ok(len) = file.read(&mut buf) {
//         if len == 9 {
//             if let Ok(id) = super::device::device_id(&buf) {
//                 return Some(id);
//             }
//         }
//     }
//     None
// }

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
            // debugln!("fn:{} ?= {}",x.file_name(), filename);
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
    Ok(fsapi::Metadata::from_dir_entry(entry, path))
}

/// 列出目录下的文件
pub fn list(path: &str) -> Result<Vec<FileEntry>, FileError> {
    let lock = DATA_DISK_FS.lock();
    let entry = seekpath(path, lock.root_dir())?;
    if !entry.is_dir() {
        return Err(FileError::NotADirError);
    }

    let result: Vec<FileEntry> = entry.to_dir()
        .iter()
        .filter(|dir_entry| { dir_entry.is_ok() })
        .map(|dir_entry| {
            let dir_entry = dir_entry.unwrap();
            let new_path = path_combine(path, dir_entry.file_name().as_str());
            if dir_entry.is_dir() { FileEntry::Dir(fsapi::Metadata::from_dir_entry(dir_entry, new_path.as_str())) } else { FileEntry::File(fsapi::Metadata::from_dir_entry(dir_entry, new_path.as_str())) }
        })
        .collect();
    Ok(result)
}

/// 获取当前工作路径
pub fn current_dir() -> String {
    proc::dir().clone()
}

/// 更换路径
pub fn change_dir(path: &str) -> Result<(), FileError> {
    let path = fsapi::path_standardize(path)?;
    let meta = metadata(path.as_str())?;
    if !meta.is_dir() {
        Err(NotADirError)
    } else {
        set_dir(path.as_str());
        Ok(())
    }
}

/// 打开文件句柄
#[derive(Clone, Debug)]
pub struct OpenFileHandle {
    pub id: usize,
    pub path: String,
    pub write: bool,
    pub device: bool,
}

/// 系统文件表-条目
pub struct SysyemFileEntry {
    pub path: String,
    pub share: usize,
    pub mutex: bool,
}

static USER_FILE_HANDLER_ID: AtomicUsize = AtomicUsize::new(4);

lazy_static! {
    static ref SYSTEM_FILE_TABLE: Mutex<BTreeMap<String,SysyemFileEntry>> = Mutex::new(BTreeMap::new());
}

fn register_opened_file(path: String, write: bool, device: bool) -> Result<usize, FileError> {
    let mut lock = SYSTEM_FILE_TABLE.lock();
    if let Some(sft) = lock.get_mut(path.as_str()) {
        if sft.mutex { Err(FileError::FileBusyError) } else {
            let fh = proc::file_handles();
            let new_id = USER_FILE_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
            fh.lock().insert(new_id, OpenFileHandle {
                id: new_id,
                path,
                write,
                device,
            });
            sft.share += 1;
            Ok(new_id)
        }
    } else {
        lock.insert(path.clone(), SysyemFileEntry {
            path: path.clone(),
            share: 1,
            mutex: write,
        });
        let fh = proc::file_handles();
        let new_id = USER_FILE_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
        fh.lock().insert(new_id, OpenFileHandle {
            id: new_id,
            path,
            write,
            device,
        });
        Ok(new_id)
    }
}

/// 打开文件（内核级）
pub fn open(path: &str, write: bool) -> Result<usize, FileError> {
    let path = fsapi::path_standardize(path)?;

    // Device Check
    if is_device(path.as_str()) { return register_opened_file(path, write, true); }

    let data = metadata(path.as_str())?;
    if !data.is_file() { Err(FileError::NotAFileError) } else {
        register_opened_file(path, write, false)
    }
}

/// 关闭文件（内核）
pub fn close(id: usize) -> Result<(), FileError> {
    if id < 4 { return Err(NotFoundError); } // 不允许关闭系统设备
    let fh = proc::file_handles();
    let mut lock = fh.lock();
    if !lock.contains_key(&id) {
        return Err(NotFoundError);
    }
    let path = lock.get(&id).unwrap().path.clone();
    lock.remove(&id);
    let mut lock = SYSTEM_FILE_TABLE.lock();
    let sft = lock.get_mut(path.as_str());
    if sft.is_none() { return Err(OSError); }
    let sft = sft.unwrap();
    if sft.share == 1 {
        lock.remove(path.as_str());
        Ok(())
    } else {
        sft.share -= 1;
        Ok(())
    }
}

fn write_all_path(path: &str, buf: &[u8]) -> Result<usize, FileError> {
    let lock = DATA_DISK_FS.lock();
    let root = lock.root_dir();
    let file = seekpath(path, root)?;
    if !file.is_file() { return Err(NotAFileError); }
    let mut file = file.to_file();

    if file.seek(SeekFrom::Start(0)).is_err() { return Err(OSError); }
    match file.write_all(buf) {
        Err(_) => Err(OSError),
        Ok(()) => Ok(buf.len())
    }
}

fn write_all_device(path: &str, buf: &[u8]) -> Result<usize, FileError> {
    super::device::write(path, buf)
}

/// 全部写
/// FIXME：暂不提供部分写、指定指针等功能
pub fn write_all(id: usize, buf: &[u8]) -> Result<usize, FileError> {
    let fh = file_handles();
    let fh_lock = fh.lock();
    if fh_lock.contains_key(&id) {
        let handle = fh_lock.get(&id).unwrap();
        if handle.write {
            if handle.device {
                write_all_device(handle.path.as_str(), buf)
            } else {
                write_all_path(handle.path.as_str(), buf)
            }
        } else { Err(FileError::OpenMethodError) }
    } else {
        Err(NotFoundError)
    }
}

/// 全部写（必须已经打开文件）
/// FIXME：暂不提供部分写、指定指针等功能
pub fn write_with_path(path: &str, buf: &[u8]) -> Result<usize, FileError> {
    let path = fsapi::path_standardize(path)?;
    let fh = file_handles();
    let fh_lock = fh.lock();
    if let Some(handle) = fh_lock.iter().find(|x| { (*x).1.path == path }) {
        if handle.1.write {
            if handle.1.device {
                write_all_device(path.as_str(), buf)
            } else {
                write_all_path(path.as_str(), buf)
            }
        } else { Err(FileError::OpenMethodError) }
    } else {
        Err(NotFoundError)
    }
}

fn read_path(path: &str, store: &mut [u8]) -> Result<usize, FileError> {
    let lock = DATA_DISK_FS.lock();
    let root = lock.root_dir();
    let file = seekpath(path, root)?;
    if !file.is_file() { return Err(NotAFileError); }
    let mut file = file.to_file();

    if file.seek(SeekFrom::Start(0)).is_err() { return Err(OSError); }
    let mut buf = [0u8; 8192];
    let mut pos = 0usize;
    while let Ok(len) = file.read(&mut buf) {
        if len == 0 {
            return Ok(pos);
        } else {
            store[pos..pos + len].copy_from_slice(&buf[0..len]);
            pos += len;
        }
    }
    Err(OSError)
}

fn read_device(path: &str, buf: &mut [u8]) -> Result<usize, FileError> {
    super::device::read(path, buf)
}

pub fn read(id: usize, buf: &mut [u8]) -> Result<usize, FileError> {
    let fh = file_handles();
    let fh_lock = fh.lock();
    if fh_lock.contains_key(&id) {
        let handle = fh_lock.get(&id).unwrap();
        if handle.device {
            read_device(handle.path.as_str(), buf)
        } else {
            read_path(handle.path.as_str(), buf)
        }
    } else {
        Err(NotFoundError)
    }
}

pub fn read_with_path(path: &str, buf: &mut [u8]) -> Result<usize, FileError> {
    let path = fsapi::path_standardize(path)?;
    let fh = file_handles();
    let fh_lock = fh.lock();
    if let Some(handle) = fh_lock.iter().find(|x| { (*x).1.path == path }) {
        if handle.1.device {
            read_device(path.as_str(), buf)
        } else {
            read_path(path.as_str(), buf)
        }
    } else {
        Err(NotFoundError)
    }
}

pub fn info(path: &str) -> Result<Metadata, FileError> {
    let path = fsapi::path_standardize(path)?;
    metadata(path.as_str())
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
    fn test_path_standardize() {
        use super::fsapi::path_standardize;
        // 测试路径为小写字母的情况
        let path = "/my_folder/my_file.txt";
        let result = path_standardize(path).unwrap();
        assert_eq!(result, "/MY_FOLDER/MY_FILE.TXT");

        // 测试路径为大写字母的情况
        let path = "/MY_FOLDER/MY_FILE.TXT";
        let result = path_standardize(path).unwrap();
        assert_eq!(result, "/MY_FOLDER/MY_FILE.TXT");

        // 测试路径包含特殊字符的情况
        let path = "/folder1/folder2/my_file.txt";
        let result = path_standardize(path).unwrap();
        assert_eq!(result, "/FOLDER1/FOLDER2/MY_FILE.TXT");

        // 测试路径包含相对路径的情况
        let path = "/folder1/./../folder2/my_file.txt";
        let result = path_standardize(path).unwrap();
        assert_eq!(result, "/FOLDER2/MY_FILE.TXT");

        // 测试空路径的情况
        let path = "";
        let result = path_standardize(path).unwrap();
        assert_eq!(result, "");
        println!("[ok]  FileSystem API test_path_standardize")
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
