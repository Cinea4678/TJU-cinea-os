use alloc::string::String;

pub trait FileIO {
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

