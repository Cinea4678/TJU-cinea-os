#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use cinea_os_sysapi::{allocator, entry_point, syscall};
use cinea_os_sysapi::call::{syscall_deserialized, syscall_deserialized_prepare, syscall_serialized, TEST_SERDE};
use cinea_os_userspace::print;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

fn main(_args: &[&str]) {
    let nowdir = String::from("~");

    let obj = cinea_os_sysapi::call::_TestSerde {
        message: alloc::string::String::from("I am from userspace."),
        number: 888,
    };

    print!("以下是将要被发送到内核的（结构体）数据：\n\tmessage: {}\n\tnumber: {}\n", obj.message.as_str(), obj.number);
    let ptr = syscall_serialized(&obj);
    let ptr_back = unsafe { syscall!(TEST_SERDE, ptr) };
    let vec_data = syscall_deserialized_prepare(ptr_back);
    let obj_back: cinea_os_sysapi::call::_TestSerde = syscall_deserialized(&vec_data).unwrap();
    print!("以下是内核返回的（结构体）数据：\n\tmessage: {}\n\tnumber: {}\n", obj_back.message.as_str(), obj_back.number);
    print!("测试结束，程序继续运行！\n");

    loop {
        print!("{} $ ", nowdir.as_str());


        loop {}
    }
}
