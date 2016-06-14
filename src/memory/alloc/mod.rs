pub mod first_fit;

use super::paging::{Page, PhysicalPage, FrameRange};
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

/// An `Allocator` implements a particular memory allocation strategy.
pub trait Allocator {

    /// Allocate a new block of size `size` on alignment `align`.
    ///
    /// # Arguments:
    ///   - `size`: the amount of memory to allocate (in bytes)
    ///   - `align`: the alignment for the allocation request
    ///
    /// # Returns:
    ///   - `Some(*mut u8)` if the request was allocated successfully
    ///   - `None` if the allocator is out of memory or if the request was
    ///     invalid.
    unsafe fn allocate( &mut self
                      , size: usize
                      , align: usize)
                      -> Option<*mut u8>;

    /// Release an allocated block of memory.
    ///
    /// The `size` and `align` parameters _must_ be the same as the original
    /// size and alignment of the frame being deallocated, otherwise our
    /// heap will become corrupted.
    ///
    /// # Arguments:
    ///   - `frame`: a pointer to the block of memory to deallocate
    ///   - `size`: the size of the block being deallocated
    ///   - `align`: the alignment of the block being deallocated
    unsafe fn deallocate( &mut self
                        , frame: *mut u8
                        , size: usize, align: usize);

    /// Reallocate `old_frame` from `old_size` bytes to `new_size` bytes
    ///
    /// The `old_size` and `align` parameters _must_ be the same as the
    /// original size and alignment of the frame being reallocated, otherwise
    /// our heap will become corrupted.
    ///
    /// # Arguments:
    ///   - `old_frame`: a pointer to the frame to be reallocated
    ///   - `old_size`: the size (in bytes) of the frame being reallocated
    ///   - `new_size`: the size to reallocate the frame to.
    ///   - `align`: the alignment for the allocation request
    ///
    /// # Returns:
    ///   - `Some(*mut u8)` if the frame was reallocated successfully
    ///   - `None` if the allocator is out of memory or if the request was
    ///     invalid.
    // TODO: Optimization: check if the reallocation request fits in
    // the old frame and return immediately if it does
    unsafe fn reallocate( &mut self
                        , old_frame: *mut u8
                        , old_size: usize
                        , new_size: usize
                        , align: usize )
                        -> Option<*mut u8> {
        // First, attempt to allocate a new frame...
        self.allocate(new_size, align)
            .map(|new_frame| {
                // If a new frame was allocated, copy all the data from the
                // old frame into the new frame.
                ptr::copy(new_frame, old_frame, cmp::min(old_size, new_size));
                // Then we can deallocate the old frame
                self.deallocate(old_frame, old_size, align);
                new_frame
            })
    }

    unsafe fn zero_alloc( &mut self
                        , _size: usize
                        , _align: usize)
                        -> Option<*mut u8> {
        unimplemented!()
    }
}
