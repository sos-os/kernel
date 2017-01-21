//! This module integrates the buddy heap allocator into the Rust runtime.
use spin::Mutex;
use core::ptr;

use ::{Allocator, FrameAllocator};
use super::{BuddyHeapAllocator, FreeList};

use memory::{ PAddr, PhysicalPage, FrameRange, VAddr };
use memory::arch::PAGE_SIZE;

pub const NUM_FREE_LISTS: usize = 19;

static ALLOC: Mutex<Option<BuddyHeapAllocator<'static>>>
    = Mutex::new(None);

static mut KERNEL_FREE_LISTS: [FreeList; NUM_FREE_LISTS]
    // TODO: I really wish there was a less awful way to do this...
    = [ FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new(),  FreeList::new(), FreeList::new()
      , FreeList::new()
      , ];

pub unsafe fn init_heap(start_addr: *mut u8, heap_size: usize ) {
    trace!(target: "alloc", "init_heap() was called.");
    *(ALLOC.lock())
        = Some(BuddyHeapAllocator::new( start_addr
                                      , &mut KERNEL_FREE_LISTS
                                      , heap_size));
}

// -- integrate the heap allocator into the Rust runtime ------------------
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    trace!("__rust_allocate() was called.");
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot allocate memory, no system allocator exists!")
             .allocate(size, align)
             .map(|blck| {
                 trace!( target: "alloc"
                       , "__rust_allocate: allocatedd {:?}", blck);
                 blck })
             .unwrap_or(ptr::null_mut())
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
             .unwrap_or(ptr::null_mut())
     }
}

/// This is currently unsupported, so we just silently ignore it
/// and return the old size.
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace( _ptr: *mut u8
                                           , old_size: usize
                                           , _: usize, _: usize )
                                           -> usize {
    old_size
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}

// quick first pass on using the heap allocator as a frame allocator
// TODO: this is Extremely Bad And Ugly And Awful. pleae make better.
//       – eliza, 1/21/2017
pub struct BuddyFrameAllocator;

impl BuddyFrameAllocator {
    pub const fn new() -> Self { BuddyFrameAllocator }
}

impl FrameAllocator for BuddyFrameAllocator {

    unsafe fn allocate(&self) -> Option<PhysicalPage> {
        ALLOC.lock().as_mut()
             .expect("Cannot allocate frame, no system allocator exists!")
             .allocate(PAGE_SIZE as usize, PAGE_SIZE as usize)
             .map(|block| {
                let addr = VAddr::from_ptr(block);
                // TODO: make this not be bad and ugly.
                PhysicalPage::containing_addr(
                    PAddr::from(addr.as_usize() as u64))
             })

    }

    unsafe fn deallocate(&self, frame: PhysicalPage) {
        ALLOC.lock().as_mut()
             .expect("Cannot deallocate frame, no system allocator exists!")
             .deallocate( frame.as_mut_ptr()
                        , PAGE_SIZE as usize
                        , PAGE_SIZE as usize);

    }

    unsafe fn allocate_range(&self, _num: usize) -> Option<FrameRange> {
        unimplemented!()
    }

    unsafe fn deallocate_range(&self, range: FrameRange) {
        for frame in range {
            ALLOC.lock().as_mut()
                 .expect("Cannot deallocate frames, no system allocator exists")
                 .deallocate( frame.as_mut_ptr()
                            , PAGE_SIZE as usize
                            , PAGE_SIZE as usize);
        }
    }


}
