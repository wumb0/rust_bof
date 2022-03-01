#![no_std]
#![feature(core_intrinsics)]

use bofhelper::import_function;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicIsize, Ordering};
use windows_sys::Win32::{
    Foundation::{BOOL, HANDLE},
    System::Memory::{HEAP_GROWABLE, HEAP_ZERO_MEMORY},
};

import_function!(NTDLL!RtlCreateHeap(Flags: u32, HeapBase: *mut u8, ReserveSize: usize, CommitSize: usize, Lock: *mut u8, Parameters: *mut u8) -> HANDLE);
import_function!(NTDLL!RtlAllocateHeap(hHeap: HANDLE, dwFlags: u32, dwByes: usize) -> *mut u8);
import_function!(NTDLL!RtlFreeHeap(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8) -> BOOL);
import_function!(NTDLL!RtlReAllocateHeap(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8, dwBytes: usize) -> *mut u8);
import_function!(NTDLL!RtlDestroyHeap(hHeap: HANDLE) -> HANDLE);

#[global_allocator]
#[link_section = ".data"]
pub static mut ALLOCATOR: BofAlloc = BofAlloc::new_uninitialized();

pub struct BofAlloc(AtomicIsize);
unsafe impl Send for BofAlloc {}
unsafe impl Sync for BofAlloc {}

#[no_mangle]
fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let lay = Layout::from_size_align_unchecked(size, align);
        ALLOCATOR.alloc(lay)
    }
}

#[no_mangle]
fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        let lay = Layout::from_size_align_unchecked(size, align);
        ALLOCATOR.dealloc(ptr, lay)
    };
}

#[no_mangle]
fn __rust_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8 {
    unsafe {
        let lay = Layout::from_size_align_unchecked(old_size, align);
        ALLOCATOR.realloc(ptr, lay, new_size)
    }
}

#[no_mangle]
fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let lay = Layout::from_size_align_unchecked(size, align);
        ALLOCATOR.alloc_zeroed(lay)
    }
}

#[no_mangle]
fn __rust_alloc_error_handler() -> ! {
    core::intrinsics::abort()
}

#[no_mangle]
fn rust_oom() -> ! {
    core::intrinsics::abort()
}

impl BofAlloc {
    pub const fn new_uninitialized() -> BofAlloc {
        BofAlloc(AtomicIsize::new(0))
    }

    #[inline]
    fn raw_handle(&self) -> HANDLE {
        self.0.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn initialize(&mut self) {
        let hh = unsafe { RtlCreateHeap(HEAP_GROWABLE, null_mut(), 0, 0, null_mut(), null_mut()) };
        self.0.store(hh, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }

    unsafe fn init_if_required(&mut self) {
        if !ALLOCATOR.is_initialized() {
            ALLOCATOR.initialize();
        }
    }

    #[inline]
    pub unsafe fn destroy(&mut self) {
        if ALLOCATOR.is_initialized() {
            // will return 0 on success
            RtlDestroyHeap(self.0.swap(0, Ordering::SeqCst));
        }
    }
}

unsafe impl GlobalAlloc for BofAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.init_if_required();
        RtlAllocateHeap(self.raw_handle(), 0, layout.size())
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.init_if_required();
        RtlAllocateHeap(self.raw_handle(), HEAP_ZERO_MEMORY, layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ALLOCATOR.init_if_required();
        RtlFreeHeap(self.raw_handle(), 0, ptr);
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATOR.init_if_required();
        RtlReAllocateHeap(self.raw_handle(), 0, ptr, new_size)
    }
}
