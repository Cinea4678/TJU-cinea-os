use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts_mut;
use core::sync::atomic::Ordering;

use embedded_graphics::pixelcolor::raw::RawU24;
use embedded_graphics::pixelcolor::Rgb888;

use cinea_os_sysapi::ExitCode;
use cinea_os_sysapi::syscall::PanicInfo;
use cinea_os_sysapi::window::WindowGraphicMemory;

use crate::{debugln, print, println, syscall_deserialize, syscall_serialized_ret, syskrnl};
use crate::syskrnl::gui::font;
use crate::syskrnl::proc::Process;

pub fn exit(code: ExitCode) -> ExitCode {
    syskrnl::proc::exit();
    code
}

pub fn sleep(seconds: f64) {
    syskrnl::time::sleep(seconds);
}

/// FIXME 在未来，要改正。现在是测试用途
pub fn spawn(number: usize, args_ptr: usize, args_len: usize, args_cap: usize) -> ExitCode {
    debugln!("{:#x},{}",args_ptr,args_len);
    let subprocess: &[u8] = match number {
        0x00 => include_bytes!("../../../dsk/bin/hello"),
        0x01 => include_bytes!("../../../dsk/bin/infprint"),
        0x02 => include_bytes!("../../../dsk/bin/taffy"),
        _ => {
            println!("spawn: invalid number");
            return ExitCode::OpenError;
        }
    };
    if let Err(code) = Process::spawn(subprocess, args_ptr, args_len, args_cap) {
        code
    } else {
        ExitCode::Success
    }
}

pub fn log(msg: usize, len: usize) -> usize {
    let ptr = syskrnl::proc::ptr_from_addr(msg as u64); // cnmd不看人家源码根本想不到
    //debugln!("log: ptr:{:p} ori_ptr:{:#x}",ptr,msg);
    let msg = unsafe { core::slice::from_raw_parts(ptr, len) };
    match core::str::from_utf8(msg) {
        Err(_) => {
            println!("log: invalid utf8 string");
            1
        }
        Ok(s) => {
            print!("{}", s);
            0
        }
    }
}

pub fn alloc(size: usize, align: usize) -> usize {
    // debugln!("ALLOC proc_id:{}",syskrnl::proc::id());
    let allocator = syskrnl::proc::heap_allocator();
    if allocator.lock().free_space() < size {
        // 需要生长，计算生长的大小
        let grow_size = size - allocator.lock().free_space();
        // 对齐到页的4KB
        let grow_size = (grow_size + 0xfff) & !0xfff;
        // 生长
        syskrnl::proc::allocator_grow(grow_size);
    }
    let ptr = unsafe { allocator.lock().alloc(core::alloc::Layout::from_size_align(size, align).expect("proc mem alloc fail 5478")) };
    ptr as usize
}

pub fn free(ptr: usize, size: usize, align: usize) {
    let allocator = syskrnl::proc::heap_allocator();
    unsafe {
        let mut lock = allocator.lock();
        lock.dealloc(ptr as *mut u8, core::alloc::Layout::from_size_align(size, align).expect("proc layout fail 5472"))
    }
}

pub fn stop_schedule() {
    syskrnl::interrupts::NO_SCHEDULE.store(true, Ordering::SeqCst);
}

pub fn restart_schedule() {
    syskrnl::interrupts::NO_SCHEDULE.store(false, Ordering::SeqCst);
}

#[doc(hidden)]
pub fn test_serde(ptr: usize) -> usize {
    use cinea_os_sysapi::call::{_TestSerde, syscall_deserialized, syscall_deserialized_prepare};

    let vec_data = syscall_deserialized_prepare(ptr);
    let obj: _TestSerde = syscall_deserialized(&vec_data).unwrap();
    println!("以下是内核通过系统调用接收到的数据：\n{:?}", obj);

    let obj_to_send = _TestSerde {
        message: alloc::string::String::from("I will be send to UserSpace"),
        number: 666,
    };

    println!("以下是内核通过系统调用返回给用户进程的数据：\n{:?}", obj_to_send);
    let ptr_back = syscall_serialized_ret!(&obj_to_send);

    ptr_back
}

pub fn list(ptr: usize) -> usize {
    let obj: String = syscall_deserialize!(ptr);

    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::list(obj.as_str()));
    ptr_back
}

pub fn open(ptr: usize) -> usize {
    let obj: (String, bool) = syscall_deserialize!(ptr);

    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::open(obj.0.as_str(), obj.1));
    ptr_back
}

pub fn info(ptr: usize) -> usize {
    let obj: String = syscall_deserialize!(ptr);

    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::info(obj.as_str()));
    ptr_back
}

pub fn write_all(ptr: usize) -> usize {
    let obj: (usize, Vec<u8>) = syscall_deserialize!(ptr);
    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::write_all(obj.0, obj.1.as_slice()));
    ptr_back
}

pub fn read(ptr: usize) -> usize {
    // 这个有点复杂了
    let obj: (usize, usize, usize) = syscall_deserialize!(ptr); // 参数1：句柄，2：地址，3：长度
    let slice = unsafe { &mut *slice_from_raw_parts_mut(obj.1 as *mut u8, obj.2) };
    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::read(obj.0, slice));
    ptr_back
}

pub fn write_path(ptr: usize) -> usize {
    let obj: (String, Vec<u8>) = syscall_deserialize!(ptr);
    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::write_with_path(obj.0.as_str(), obj.1.as_slice()));
    ptr_back
}

pub fn read_path(ptr: usize) -> usize {
    // 这个有点复杂了
    let obj: (String, usize, usize) = syscall_deserialize!(ptr); // 参数1：文件路径，2：地址，3：长度
    let slice = unsafe { &mut *slice_from_raw_parts_mut(obj.1 as *mut u8, obj.2) };
    let ptr_back = syscall_serialized_ret!(&syskrnl::fs::read_with_path(obj.0.as_str(), slice));
    ptr_back
}

pub fn panic(ptr: usize) -> usize {
    let obj: PanicInfo = syscall_deserialize!(ptr);
    println!("{:?}", obj);
    panic!("User-Space APP asked to panic. ACCR");
}

pub fn create_window(ptr: usize) -> usize {
    let obj: (String, usize) = syscall_deserialize!(ptr);
    syscall_serialized_ret!(&syskrnl::gui::WINDOW_MANAGER.lock().create_window(obj.0.as_str(),obj.1))
}

pub fn display_font_string(ptr: usize) -> usize {
    let obj: (usize, String, String, usize, usize, f32, usize, u32) = syscall_deserialize!(ptr);
    let window = unsafe { &mut *(obj.0 as *mut WindowGraphicMemory) };
    let color = Rgb888::from(RawU24::new(obj.7));
    font::display_font_string(window, obj.1.as_str(), obj.2.as_str(), obj.3, obj.4, obj.5, obj.6, color);
    0
}

pub fn load_font(ptr: usize) -> usize {
    let obj: (String, String) = syscall_deserialize!(ptr);
    let ret = font::load_font(obj.0.as_str(),obj.1.as_str());
    syscall_serialized_ret!(&ret.is_ok())
}