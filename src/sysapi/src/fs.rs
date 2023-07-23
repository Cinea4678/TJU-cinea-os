use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::Add;
use core::ptr::slice_from_raw_parts;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use FileError::BadRelatePathError;
use crate::call::{LIST, LOG};
use crate::syscall;

use crate::time::{Date, DateTime};

pub trait FileIO: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()>;
}

/// Returns the directory component of a pathname.
///
/// Given a pathname, this function returns the leading component of the pathname, up to and including the last slash `/`.
/// If there is no slash in the pathname, the entire pathname is returned.
///
/// # Examples
///
/// ```
/// use crate::fs::dirname;
///
/// assert_eq!(dirname("/usr/bin/gcc"), "/usr/bin");
/// assert_eq!(dirname("/usr/bin/"), "/usr/bin");
/// assert_eq!(dirname("/usr/"), "/usr");
/// assert_eq!(dirname("/"), "/");
/// assert_eq!(dirname("gcc"), "gcc");
/// ```
///
/// # Arguments
///
/// * `pathname`: A string slice containing the pathname to extract the directory component from.
///
/// # Returns
///
/// The directory component of the pathname.
///
/// # Safety
///
/// This function is safe to use as long as the `pathname` argument is a valid null-terminated string.
/// If the `pathname` argument is not a valid null-terminated string, this function may cause undefined behavior.
pub fn dirname(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(0) => 1, // 根目录
        Some(i) => i,
        None => n,
    };
    &pathname[0..i]
}

/// Returns the filename component of a pathname.
///
/// Given a pathname, this function returns the trailing component of the pathname, after the last slash `/`.
/// If there is no slash in the pathname, the entire pathname is returned.
///
/// # Examples
///
/// ```
/// use crate::fs::filename;
///
/// assert_eq!(filename("/usr/bin/gcc"), "gcc");
/// assert_eq!(filename("/usr/bin/"), "");
/// assert_eq!(filename("/usr/"), "");
/// assert_eq!(filename("/"), "");
/// assert_eq!(filename("gcc"), "gcc");
/// ```
///
/// # Arguments
///
/// * `pathname`: A string slice containing the pathname to extract the filename component from.
///
/// # Returns
///
/// The filename component of the pathname.
///
/// # Safety
///
/// This function is safe to use as long as the `pathname` argument is a valid null-terminated string.
/// If the `pathname` argument is not a valid null-terminated string, this function may cause undefined behavior.
pub fn filename(pathname: &str) -> &str {
    let n = pathname.len();
    let i = match pathname.rfind('/') {
        Some(i) => i + 1,
        None => 0,
    };
    &pathname[i..n]
}

/// Returns the absolute path of a pathname.
///
/// Given a pathname and the current working directory, this function returns the absolute path of the pathname.
/// If the pathname is already an absolute path, it is returned unchanged.
/// Otherwise, the pathname is resolved relative to the current working directory.
///
/// # Examples
///
/// ```
/// use crate::fs::realpath;
///
/// assert_eq!(realpath("/usr/bin/gcc", "/home/user"), "/usr/bin/gcc");
/// assert_eq!(realpath("gcc", "/usr/bin"), "/usr/bin/gcc");
/// assert_eq!(realpath("gcc", "/usr/bin/"), "/usr/bin/gcc");
/// ```
///
/// # Arguments
///
/// * `pathname`: A string slice containing the pathname to resolve.
/// * `current_dir`: A string slice containing the current working directory to resolve the pathname relative to.
///
/// # Returns
///
/// The absolute path of the pathname.
///
/// # Safety
///
/// This function is safe to use as long as the `pathname` and `current_dir` arguments are valid null-terminated strings.
/// If the `pathname` or `current_dir` arguments are not valid null-terminated strings, this function may cause undefined behavior.
pub fn realpath(pathname: &str, current_dir: &str) -> String {
    if pathname.starts_with('/') {
        pathname.into()
    } else {
        let sep = if current_dir.ends_with('/') { "" } else { "/" };
        alloc::format!("{}{}{}", current_dir, sep, pathname)
    }
}

#[derive(Debug)]
pub enum FileError {
    NotFoundError,
    /// Root directory cannot be used for some operations.
    RootDirError,
    BadRelatePathError,
    /// **Not a Dir Error**, seen in operations must be executed to directories.
    NotADirError,
    /// Seen in reading or writing device files.
    DeviceIOError,
}

bitflags! {
    /// A FAT file attributes.
    #[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
    pub struct FileAttributes: u8 {
        const READ_ONLY  = 0x01;
        const HIDDEN     = 0x02;
        const SYSTEM     = 0x04;
        const VOLUME_ID  = 0x08;
        const DIRECTORY  = 0x10;
        const ARCHIVE    = 0x20;
        const LFN        = Self::READ_ONLY.bits() | Self::HIDDEN.bits()
                         | Self::SYSTEM.bits() | Self::VOLUME_ID.bits();
    }
}


/// Represents metadata information for a file or directory.
///
/// # Examples:
///
/// ```
/// use your_operating_system::Metadata;
///
/// fn main() {
///     let metadata = Metadata {
///         short_file_name: "file.txt".to_string(),
///         file_name: "file.txt".to_string(),
///         attributes: FileAttributes::default(),
///         is_dir: false,
///         is_file: true,
///         len: 1024,
///         created: DateTime::now(),
///         accessed: Date::today(),
///         modified: DateTime::now(),
///     };
///
///     println!("Short file name: {}", metadata.short_file_name());
///     println!("Full file name: {}", metadata.file_name());
///     println!("Is directory: {}", metadata.is_dir());
///     println!("Is file: {}", metadata.is_file());
///     println!("File size: {} bytes", metadata.len());
///     println!("Creation date: {}", metadata.created());
///     println!("Last access date: {}", metadata.accessed());
///     println!("Last modification date: {}", metadata.modified());
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Absolute Path
    path: String,
    /// Short file name
    short_file_name: String,
    /// Full file name
    file_name: String,
    /// File attributes
    attributes: FileAttributes,
    /// Indicates if it is a directory
    is_dir: bool,
    /// Indicates if it is a regular file
    is_file: bool,
    /// Size of the file in bytes
    len: u64,
    /// Creation date and time
    created: DateTime,
    /// Last access date
    accessed: Date,
    /// Last modification date and time
    modified: DateTime,
}

impl Metadata {
    /// Initialize the object with DirEntry
    pub fn from_dir_entry<'a, IO, TP, OCC>(entry: fatfs::DirEntry<'a, IO, TP, OCC>, path: &str) -> Self
        where IO: fatfs::ReadWriteSeek, OCC: fatfs::OemCpConverter {
        Self {
            path: String::from(path),
            short_file_name: entry.short_file_name(),
            file_name: entry.file_name(),
            attributes: FileAttributes::from_bits_retain(entry.attributes().bits()),
            is_dir: entry.is_dir(),
            is_file: entry.is_file(),
            len: entry.len(),
            created: DateTime::from_fatfs(&entry.created()),
            accessed: Date::from_fatfs(&entry.accessed()),
            modified: DateTime::from_fatfs(&entry.modified()),
        }
    }

    /// Returns the short file name.
    pub fn short_file_name(&self) -> &str {
        &self.short_file_name
    }

    /// Returns the full file name.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Returns the file attributes.
    pub fn attributes(&self) -> FileAttributes {
        self.attributes
    }

    /// Returns true if the metadata represents a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    /// Returns true if the metadata represents a regular file.
    pub fn is_file(&self) -> bool {
        self.is_file
    }

    /// Returns the size of the file in bytes.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns the creation date and time.
    pub fn created(&self) -> DateTime {
        self.created
    }

    /// Returns the last access date.
    pub fn accessed(&self) -> Date {
        self.accessed
    }

    /// Returns the last modification date and time.
    pub fn modified(&self) -> DateTime {
        self.modified
    }
}

pub fn process_relative_path(splited_path: &mut Vec<&str>) -> Result<(), FileError> {
    let mut i = 0usize;
    while i < splited_path.len() {
        if splited_path[i] == "." {
            splited_path.remove(i);
        } else if splited_path[i] == ".." {
            if i == 0 { return Err(BadRelatePathError) }
            splited_path.remove(i);
            splited_path.remove(i - 1);
            i -= 1;
        } else {
            i += 1;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDevice(usize);

impl FileDevice {}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileEntry {
    Dir(Metadata),
    File(Metadata),
    Device(FileDevice),
}

impl FileEntry {
    pub fn new_dir(metadata: Metadata) -> Self {
        Self::Dir(metadata)
    }

    pub fn new_file(metadata: Metadata) -> Self {
        Self::File(metadata)
    }

    pub fn new_device(device: usize) -> Self {
        Self::Device(FileDevice(device))
    }

    pub fn list(&mut self) -> Result<Vec<Self>, FileError> {
        match self {
            FileEntry::Dir(dir) => {
                // 调用系统调用查询
                let encoded = postcard::to_allocvec(dir).unwrap();

                let ret = unsafe { syscall!(LIST,encoded.as_ptr() as usize) };

                let result_ptr = ret as *const u64;
                let result_len = unsafe { *result_ptr };
                let result_raw = unsafe { &*slice_from_raw_parts(ret.add(1) as *const u8, result_len as usize) };

                // let decoded: Result<Result<Vec<Self>, FileError>, _> = postcard::from_bytes(result_raw);
                unimplemented!();


                // Ok(vec![])
            },
            _ => { Err(FileError::NotADirError) }
        }
    }
}

pub fn info(path: &str) -> FileEntry {
    todo!()
}