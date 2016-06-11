use super::paging::{Page, PageRange};
use core::ops;

/// A borrowed handle on a frame with a specified lifetime.
///
/// This automatically deallocates the frame when the borrow's lifetime
/// ends. It also ensures that the borrow only lives as long as the allocator
/// that provided it, and that the borrow is dropped if the allocator is
/// dropped.
pub struct BorrowedFrame<'a, F, A>
where F: Page
    , A: FrameAllocator<F>
    , A: 'a {
    frame: F
  , allocator: &'a A
}

impl<'a, F, A> ops::Deref for BorrowedFrame<'a, F, A>
where F: Page
    , A: 'a {
    type Target = F;
    fn deref(&self) -> &F { &self.frame }
}

impl<'a, F, A> ops::DerefMut for BorrowedFrame<'a, F, A>
where F: Page
    , A: 'a {
    fn deref_mut(&mut self) -> &mut F { &mut self.frame }
}

impl<'a, F, A> Drop for BorrowedFrame<'a, F, A>
where F: Page
    , A: FrameAllocator<F>
    , A: 'a {
    fn drop(&mut self) {
        unsafe { self.allocator.deallocate(self.frame) }
    }
}

/// Identical to a `BorrowedFrame` but borrowing a range of `Frame`s.
pub struct BorrowedFrameRange<'a, F, A>
where F: Page
    , A: FrameAllocator<F>
    , A: 'a {
    range: PageRange<F>
  , allocator: &'a A
}

impl<'a, F, A> ops::Deref for BorrowedFrame<'a, F, A>
where F: Page
    , A: 'a {
    type Target = PageRange<F>;
    fn deref(&self) -> &Self::Target { &self.range }
}

impl<'a, F, A> ops::DerefMut for BorrowedFrame<'a, F, A>
where F: Page
    , A: 'a {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.range }
}

impl<'a, F, A> Drop for BorrowedFrame<'a, F, A>
where F: Page
    , A: FrameAllocator<F>
    , A: 'a {
    fn drop(&mut self) {
        unsafe { self.allocator.deallocate_range(self.range) }
    }
}


pub trait FrameAllocator<Frame>
where Frame: Page {

    unsafe fn allocate(&self) -> Option<Frame>;
    unsafe fn deallocate(&self, frame: Frame);
    fn borrow(&self) -> Option<BorrowedFrame<Frame, Self>>;

    unsafe fn allocate_range(&self, num: usize) -> Option<PageRange<Frame>>;
    unsafe fn deallocate_range(&self, range: PageRange<Frame>);
    fn borrow_range(&self, num: usize)
                    -> Option<BorrowedFrameRange<Frame, Self>>;


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
                ptr::copy(new_frame, old_frame, min(old_size, new_size));
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
