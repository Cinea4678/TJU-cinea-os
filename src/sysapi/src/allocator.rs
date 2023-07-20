//! This module provides an allocator for user processes.
//!
//! The `UserProcAllocator` struct implements the `GlobalAlloc` trait, which allows it to be used as a global allocator for Rust's memory allocation functions.
//!
//! # Examples
//!
//! ```
//! use crate::allocator::UserProcAllocator;
//! use std::alloc::{alloc, dealloc, Layout};
//!
//! let allocator = UserProcAllocator;
//! let layout = Layout::from_size_align(16, 4).unwrap();
//!
//! let ptr = unsafe { allocator.alloc(layout) };
//! assert!(!ptr.is_null());
//!
//! unsafe { allocator.dealloc(ptr, layout) };
//! ```
//!
//! # Safety
//!
//! This allocator is unsafe because it relies on the `alloc` and `free` system calls provided by the operating system.
//! It is the responsibility of the caller to ensure that the memory being allocated and deallocated is valid and that the system calls are safe to make.
//! Additionally, the `dealloc` function assumes that the `ptr` argument was previously allocated by the same allocator and with the same layout, so it is up to the caller to ensure that this is the case.
//!
//! # Note
//!
//! This allocator is intended for use in user processes only.

use core::alloc::{GlobalAlloc, Layout};

/// Userspace process heap memory allocator
pub struct UserProcAllocator;

unsafe impl GlobalAlloc for UserProcAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::syscall::alloc(layout.size(), layout.align()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::syscall::free(ptr as usize, layout.size(), layout.align());
    }
}