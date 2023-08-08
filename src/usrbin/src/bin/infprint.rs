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
    if args.len() > 1 {
        let mut num = 0;
        let output = String::from(args[0]);
        let sleep_time = usize::from_str_radix(args[1], 10).unwrap();
        uwrite!(
            strout,
            "{}, 我的sleepTime是{}\n",
            output.as_str(),
            sleep_time
        )
        .unwrap();
        loop {
            uwrite!(strout, "{}, 我已经输出了{}次\n", output.as_str(), num).unwrap();
            log(strout.value().as_bytes());
            strout.clear();
            cinea_os_sysapi::event::sleep(sleep_time);
            num += 1;
        }
    }
}
