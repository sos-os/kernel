pub mod first_fit;

use super::{PAGE_SIZE, PAddr, VAddr};
use super::paging::{Page, PhysicalPage, FrameRange};

use alloc::buddy::BuddyHeapAllocator;
use alloc::buddy::system::ALLOC;
use alloc::Allocator;

use core::{ops, ptr, cmp};

/// A borrowed handle on a frame with a specified lifetime.
///
/// This automatically deallocates the frame when the borrow's lifetime
/// ends. It also ensures that the borrow only lives as long as the allocator
/// that provided it, and that the borrow is dropped if the allocator is
/// dropped.
pub struct BorrowedFrame<'a, A>
where A: FrameAllocator
    , A: 'a {
    frame: PhysicalPage
  , allocator: &'a A
}

impl<'a, A> ops::Deref for BorrowedFrame<'a, A>
where A: FrameAllocator
    , A: 'a {
    type Target = PhysicalPage;
    fn deref(&self) ->  &Self::Target { &self.frame }
}

impl<'a, A> ops::DerefMut for BorrowedFrame<'a, A>
where A: FrameAllocator
    , A: 'a {
    fn deref_mut(&mut self) ->  &mut Self::Target { &mut self.frame }
}

impl<'a, A> Drop for BorrowedFrame<'a, A>
where A: FrameAllocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe { self.allocator.deallocate(self.frame) }
    }
}

/// Identical to a `BorrowedFrame` but borrowing a range of `Frame`s.
pub struct BorrowedFrameRange<'a, A>
where A: FrameAllocator
    , A: 'a {
    range: FrameRange
  , allocator: &'a A
}

impl<'a, A> ops::Deref for BorrowedFrameRange<'a, A>
where A: FrameAllocator
    , A: 'a {
    type Target = FrameRange;
    fn deref(&self) -> &Self::Target { &self.range }
}

impl<'a, A> ops::DerefMut for BorrowedFrameRange<'a, A>
where A: FrameAllocator
    , A: 'a {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.range }
}

impl<'a, A> Drop for BorrowedFrameRange<'a, A>
where A: FrameAllocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe { self.allocator.deallocate_range(self.range) }
    }
}


pub trait FrameAllocator: Sized  {

    unsafe fn allocate(&self) -> Option<PhysicalPage>;
    unsafe fn deallocate(&self, frame: PhysicalPage);

    /// Borrow a `Frame` from this allocator.
    ///e
    /// The `BorrowedFrame` will live as long as this allocator, and will
    /// contain a handle on a `Frame` that will be automatically deallocated
    /// when the `BorrowedFrame` is dropped.
    ///
    /// # Returns:
    /// + `Some(BorrowedFrame)` if there are frames remaining in this
    ///    allocator.
    /// + `None` if the allocator is out of frames.
    fn borrow(&self) -> Option<BorrowedFrame<Self>> {
        unsafe { self.allocate() }
                     .map(|frame| BorrowedFrame { frame: frame
                                                , allocator: self })
    }

    unsafe fn allocate_range(&self, num: usize) -> Option<FrameRange>;
    unsafe fn deallocate_range(&self, range: FrameRange);

    /// Borrow a `FrameRange` from this allocator.
    ///
    /// The `BorrowedFrameRange` will live as long as this allocator, and will
    /// contain a handle on a range of `Frame`s that will be automatically
    /// deallocated when the `BorrowedFrameRange` is dropped.
    ///
    /// # Arguments:
    /// + `num`: The number of frames to allocate.
    ///
    /// # Returns:
    /// + `Some(BorrowedFrameRange)` if there are enough `Frame`s
    ///    remaining in the allocator to fulfill the allocation
    ///    request.
    /// + `None` if there are not enough frames in the allocator to fulfill the
    ///   allocation request.
    fn borrow_range(&self, num: usize) -> Option<BorrowedFrameRange<Self>> {
        unsafe { self.allocate_range(num) }
                     .map(|range| BorrowedFrameRange { range: range
                                                     , allocator: self })
    }


}

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
        for frame in range.iter() {
            ALLOC.lock().as_mut()
                 .expect("Cannot deallocate frames, no system allocator exists")
                 .deallocate( frame.as_mut_ptr()
                            , PAGE_SIZE as usize
                            , PAGE_SIZE as usize);
        }
    }


}
