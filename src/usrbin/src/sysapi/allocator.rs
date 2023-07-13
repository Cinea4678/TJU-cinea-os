use core::alloc::{GlobalAlloc, Layout};
use crate::sysapi;

pub struct UserProcAllocator;

unsafe impl GlobalAlloc for UserProcAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        sysapi::syscall::alloc(layout.size(), layout.align()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        sysapi::syscall::free(ptr as usize, layout.size(), layout.align());
    }
}