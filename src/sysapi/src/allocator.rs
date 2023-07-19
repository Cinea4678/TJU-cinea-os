use core::alloc::{GlobalAlloc, Layout};

pub struct UserProcAllocator;

unsafe impl GlobalAlloc for UserProcAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::syscall::alloc(layout.size(), layout.align()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::syscall::free(ptr as usize, layout.size(), layout.align());
    }
}