#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::{format, vec::Vec};

use cinea_os_sysapi::{allocator, entry_point, syscall::spawn};
use cinea_os_userspace::{print, std};

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

fn main(_args: &[&str]) {
    let nowdir = String::from("~");

    print!("文件系统测试：打开文件、写入；写入设备文件。\n");
    print!("\n打开测试文件：/sys/helloworld.txt\n");
    let handle = std::fs::open("/sys/helloworld.txt", false).unwrap();
    print!("读取测试文件：");
    let mut buffer = [0u8; 20];
    std::fs::read(handle, &mut buffer).unwrap();
    let res = core::str::from_utf8(&buffer).unwrap();
    print!("{}", res);
    print!("\n\n打开测试设备（1）：/dev/stdout\n此测试设备无需打开，直接尝试写入内容：Hello File System\n");
    std::fs::write_path("/dev/stdout", b"Hello File System\n").unwrap();
    print!("\n\n打开测试设备（2）：/dev/uptime\n");
    let handle = std::fs::open("/dev/uptime", false).unwrap();
    print!("读取设备：");
    let mut buffer = [0u8; 8];
    std::fs::read(handle, &mut buffer).unwrap();
    let uptime = f64::from_le_bytes(buffer);
    print!(
        "{}\n\n现在基于进程并行测试Sleep\n",
        ryu::Buffer::new().format(uptime)
    );

    loop {
        //print!("{} $ ", nowdir.as_str());

        loop {}
    }
}
