//! This module integrates the buddy heap allocator into the Rust runtime.
use spin::Mutex;
use core::ptr;

use ::{Allocator, FrameAllocator, Layout};
use super::{HeapAllocator, FreeList};

use memory::{ PAGE_SIZE, PAddr, PhysicalPage, FrameRange, VAddr };

/// The number of free lists for the kernel heap
pub const NUM_FREE_LISTS: usize = 19;

static ALLOC: Mutex<Option<HeapAllocator<'static>>>
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

/// Initialize the system heap at the given start address
///
/// # Arguments
/// + `start_addr`: a pointer to the start address of the kernel heap
/// + `heap_size`: the maximum size (in bytes) of the kernel heap
///
/// # Panics
/// + If called once the kernel heap is already initialized
pub unsafe fn init_heap(start_addr: *mut u8, heap_size: usize ) {
    assert_has_not_been_called!("the kernel heap may not be initialized \
                                 more than once!");
    trace!(target: "alloc", "init_heap() was called.");
    *(ALLOC.lock())
        = Some(HeapAllocator::new( start_addr
                                      , &mut KERNEL_FREE_LISTS
                                      , heap_size));
}

// -- integrate the heap allocator into the Rust runtime ------------------
#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    trace!("__rust_allocate() was called.");
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot allocate memory, no system allocator exists!")
             .alloc(Layout::from_size_align(size, align))
             .map(|blck| {
                 // TODO: can we use `inspect()` here instead?
                 //       - eliza, 1/23/2017
                 trace!( target: "alloc"
                       , "__rust_allocate: allocated {:?}", blck);
                 blck })
            // TODO: how to handle various error conditions here in
            //       ways the stdlib expects?
            //          - eliza, 02/02/2017
             .unwrap()
    }
}
#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn __rust_deallocate( ptr: *mut u8, old_size: usize
                                   , align: usize ) {
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot deallocate memory, no system allocator exists!")
             .dealloc(ptr, Layout::from_size_align(old_size, align))
    }
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn __rust_reallocate( ptr: *mut u8, old_size: usize
                                   , size: usize, align: usize )
                                   -> *mut u8 {
    unsafe {
        ALLOC.lock().as_mut()
             .expect("Cannot reallocate memory, no system allocator exists!")
             .realloc( ptr
                     , Layout::from_size_align(old_size, align)
                     , Layout::from_size_align(size, align))
             // TODO: how to handle various error conditions here in
             //       ways the stdlib expects?
             //          - eliza, 02/02/2017
             .unwrap()
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

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}

/// A frame allocator using the system's buddy-block heap allocator
// quick first pass on using the heap allocator as a frame allocator
// TODO: this is Extremely Bad And Ugly And Awful. pleae make better.
//       – eliza, 1/21/2017
pub struct BuddyFrameAllocator;

impl BuddyFrameAllocator {
    /// Construct a new `BuddyFrameAllocator`
    pub const fn new() -> Self { BuddyFrameAllocator }
}
//
// impl FrameAllocator for BuddyFrameAllocator {
//
//     unsafe fn allocate(&self) -> Option<PhysicalPage> {
//         ALLOC.lock().as_mut()
//              .expect("Cannot allocate frame, no system allocator exists!")
//              .allocate(PAGE_SIZE as usize, PAGE_SIZE as usize)
//              .map(|block| {
//                 let addr = VAddr::from_ptr(block);
//                 // TODO: make this not be bad and ugly.
//                 PhysicalPage::containing_addr(
//                     PAddr::from(addr.as_usize() as u64))
//              })
//
//     }
//
//     unsafe fn deallocate(&self, frame: PhysicalPage) {
//         ALLOC.lock().as_mut()
//              .expect("Cannot deallocate frame, no system allocator exists!")
//              .deallocate( frame.as_mut_ptr()
//                         , PAGE_SIZE as usize
//                         , PAGE_SIZE as usize);
//
//     }
//
//     unsafe fn allocate_range(&self, _num: usize) -> Option<FrameRange> {
//         unimplemented!()
//     }
//
//     unsafe fn deallocate_range(&self, range: FrameRange) {
//         for frame in range {
//             ALLOC.lock().as_mut()
//                  .expect("Cannot deallocate frames, no system allocator exists")
//                  .deallocate( frame.as_mut_ptr()
//                             , PAGE_SIZE as usize
//                             , PAGE_SIZE as usize);
//         }
//     }
//
//
// }
