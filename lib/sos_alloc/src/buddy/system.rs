//! This module integrates the buddy heap allocator into the Rust runtime.
use spin::Mutex;

use ::Allocator;
use super::{BuddyHeapAllocator, FreeList};


static ALLOC: Mutex<Option<BuddyHeapAllocator<'static>>>
    = Mutex::new(None);

pub unsafe fn init_heap( start_addr: *mut u8
                       , free_lists: &'static mut [FreeList<'static>]
                       , heap_size: usize ) {
    trace!("init_heap() was called.");
    *(ALLOC.lock())
        = Some(BuddyHeapAllocator::new(start_addr, free_lists, heap_size));
}

#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    trace!("__rust_allocate() was called.");
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot allocate memory, no system allocator exists!")
             .allocate(size, align)
             .map(|blck| {
                 trace!("__rust_allocate: allocatedd {:?}", blck);
                 blck })
             .expect("Memory could not be allocated; either the allocator is\
                      out of memory, or the allocation request was invalid.")
    }
}

#[no_mangle]
pub extern "C" fn __rust_deallocate( ptr: *mut u8, old_size: usize
                                   , align: usize ) {
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot deallocate memory, no system allocator exists!")
             .deallocate(ptr, old_size, align)
    }
}


#[no_mangle]
pub extern "C" fn __rust_reallocate( ptr: *mut u8, old_size: usize
                                   , size: usize, align: usize )
                                   -> *mut u8 {
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot reallocate memory, no system allocator exists!")
             .reallocate(ptr, old_size, size, align)
             .expect("Memory could not be reallocated; either the allocator\
                     is out of memory, or the allocation request was invalid.")
     }
}

/// This is currently unsupported, so we just silently ignore it
/// and return the old size.
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace( ptr: *mut u8, old_size: usize
                                           , _size: usize, _align: usize )
                                           -> usize {
    old_size
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
