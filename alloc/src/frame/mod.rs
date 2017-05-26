//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Frame allocation
#![warn(missing_docs)]
use memory::{FrameRange, PhysicalPage as Frame};
use super::AllocResult;
use core::ops;
use spin::Mutex;

pub mod area;

/// An allocator for allocating physical frames.
pub trait Allocator: Sized  {

    /// Allocate a new frame
    unsafe fn allocate(&mut self) -> AllocResult<Frame>;
    /// Deallocate a frame
    unsafe fn deallocate(&mut self, frame: Frame);

    /// Allocate a range of frames
    unsafe fn allocate_range(&mut self, num: usize) -> AllocResult<FrameRange>;
    /// Deallocate a range of frames
    unsafe fn deallocate_range(&mut self, range: FrameRange);

}

/// An allocator capable of lending [borrowed frame]s
///
/// [borrowed frame]: struct.BorrowedFrame.html
pub trait Lender<A>
where A: Allocator {
    /// Borrow a `Frame` from this allocator.
    ///e
    /// The `BorrowedFrame` will live as long as this allocator, and will
    /// contain a handle on a `Frame` that will be automatically deallocated
    /// when the `BorrowedFrame` is dropped.
    ///
    /// # Returns
    /// + `Some(BorrowedFrame)` if there are frames remaining in this
    ///    allocator.
    /// + `None` if the allocator is out of frames.
    //  TODO: do we want to refactor this into returning Results?
    //          - eliza, 02/21/2017
    fn borrow(&self) -> AllocResult<BorrowedFrame<A>>;

    /// Borrow a `FrameRange` from this allocator.
    ///
    /// The `BorrowedFrameRange` will live as long as this allocator, and will
    /// contain a handle on a range of `Frame`s that will be automatically
    /// deallocated when the `BorrowedFrameRange` is dropped.
    ///
    /// # Arguments
    /// + `num`: The number of frames to allocate.
    ///
    /// # Returns
    /// + `Some(BorrowedFrameRange)` if there are enough `Frame`s
    ///    remaining in the allocator to fulfill the allocation
    ///    request.
    /// + `None` if there are not enough frames in the allocator to fulfill the
    ///   allocation request.
    //  TODO: do we want to refactor this into returning Results?
    //          - eliza, 02/21/2017
    fn borrow_range(&self, num: usize) -> AllocResult<BorrowedFrameRange<A>>;

}

impl<A: Allocator> Lender<A> for Mutex<A> {

    fn borrow(&self) -> AllocResult<BorrowedFrame<A>> {
        // TODO: can this be rewritten to just use `self.borrow_range(1)`?
        //          - eliza, 02/21/2017
        unsafe { self.lock().allocate() }
                     .map(|frame| BorrowedFrame { frame: frame
                                                , allocator: self })
    }

    fn borrow_range(&self, num: usize) -> AllocResult<BorrowedFrameRange<A>> {
        unsafe { self.lock().allocate_range(num) }
                     .map(|range| BorrowedFrameRange { range: range
                                                     , allocator: self })
    }



}

/// A borrowed handle on a frame with a specified lifetime.
///
/// This automatically deallocates the frame when the borrow's lifetime
/// ends. It also ensures that the borrow only lives as long as the allocator
/// that provided it, and that the borrow is dropped if the allocator is
/// dropped.
pub struct BorrowedFrame<'alloc, A>
where A: Allocator
    , A: 'alloc {
    frame: Frame
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A> ops::Deref for BorrowedFrame<'alloc, A>
where A: Allocator
    , A: 'alloc {
    type Target = Frame;
    fn deref(&self) ->  &Self::Target { &self.frame }
}

impl<'alloc, A> ops::DerefMut for BorrowedFrame<'alloc, A>
where A: Allocator
    , A: 'alloc {
    fn deref_mut(&mut self) ->  &mut Self::Target { &mut self.frame }
}

impl<'alloc, A> Drop for BorrowedFrame<'alloc, A>
where A: Allocator
    , A: 'alloc {
    fn drop(&mut self) {
        unsafe { self.allocator.lock().deallocate(self.frame) }
    }
}

/// Identical to a `BorrowedFrame` but borrowing a range of `Frame`s.
pub struct BorrowedFrameRange<'alloc, A>
where A: Allocator
    , A: 'alloc {
    range: FrameRange
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A> ops::Deref for BorrowedFrameRange<'alloc, A>
where A: Allocator
    , A: 'alloc {
    type Target = FrameRange;
    fn deref(&self) -> &Self::Target { &self.range }
}

impl<'alloc, A> ops::DerefMut for BorrowedFrameRange<'alloc, A>
where A: Allocator
    , A: 'alloc {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.range }
}

impl<'alloc, A> Drop for BorrowedFrameRange<'alloc, A>
where A: Allocator
    , A: 'alloc {
    fn drop(&mut self) {
        unsafe { self.allocator.lock().deallocate_range(self.range.clone()) }
    }
}
