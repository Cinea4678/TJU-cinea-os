use crate::{debugln, print, println, syskrnl};
use crate::syskrnl::proc::Process;
use crate::syskrnl::sysapi::ExitCode;

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
    debugln!("ALLOC proc_id:{}",syskrnl::proc::id());
    debugln!("1");
    let allocator = syskrnl::proc::heap_allocator();
    debugln!("2");
    if allocator.lock().free_space() < size {
        debugln!("3");
        // 需要生长，计算生长的大小
        let grow_size = size - allocator.lock().free_space();
        // 对齐到页的4KB
        let grow_size = (grow_size + 0xfff) & !0xfff;
        // 生长
        syskrnl::proc::allocator_grow(grow_size);
    }
    debugln!("4");
    let ptr = unsafe { allocator.lock().alloc(core::alloc::Layout::from_size_align(size, align).expect("proc mem alloc fail 5478")) };
    debugln!("5");
    ptr as usize
}

pub fn free(ptr: usize, size: usize, align: usize) {
    let allocator = syskrnl::proc::heap_allocator();
    unsafe {
        let mut lock = allocator.lock();
        lock.dealloc(ptr as *mut u8, core::alloc::Layout::from_size_align(size, align).expect("proc layout fail 5472"))
    }
}