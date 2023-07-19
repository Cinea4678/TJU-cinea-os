#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use ufmt::uwrite;

use cinea_os_sysapi::{allocator, entry_point, syscall::log};
use cinea_os_userspace::std::StringWriter;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

fn main(args: &[&str]) {
    let mut strout = StringWriter::new();
    if args.len() > 0 {
        let mut num = 0;
        let output = String::from(args[0]);
        loop {
            uwrite!(strout, "{}, 我已经输出了{}次\n", output.as_str(), num).unwrap();
            log(strout.value().as_bytes());
            strout.clear();
            for _ in 0..10000000 { // 土法sleep
            }
            num += 1;
        }
    }
}
