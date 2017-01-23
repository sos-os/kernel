//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS memory allocation library
//!
//! This is in its own crate so it can be used by kernel-space and user-space
//! OS components.
#![crate_name = "alloc"]
#![crate_type = "lib"]

// The compiler needs to be instructed that this crate is an allocator in order
// to realize that when this is linked in another allocator like jemalloc
// should not be linked in
#![cfg_attr( feature = "as_system", feature(allocator) )]
#![cfg_attr( feature = "as_system", allocator )]

#![cfg_attr( feature = "buddy", feature(unique))]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

// Allocators are not allowed to depend on the standard library which in turn
// requires an allocator in order to avoid circular dependencies. This crate,
// however, can use all of libcore.
#![no_std]

#![feature( const_fn )]

extern crate memory;
use memory::{PhysicalPage, FrameRange};

use core::{ops, ptr};
use core::cmp::min;

#[cfg(feature = "first_fit")]
extern crate arrayvec;

#[cfg(feature = "buddy")]
extern crate sos_intrusive as intrusive;

#[cfg(any( feature = "buddy_as_system"
         , feature = "first_fit"))]
extern crate spin;

#[cfg(feature = "buddy_as_system")]
#[macro_use] extern crate once;

#[macro_use] extern crate log;




/// Trait for something that is like a frame.
///
/// Various allocation strategies use different data structures for
/// representing frames. For example, frames may be stored as frame numbers or
/// as nodes in a linked list. To be `Framesque`, an object need only provide
/// a function to convert the frame data to a pointer to the frame in memory.
pub trait Framesque {
    /// Return a pointer to the frame in memory.
    fn as_ptr(&self) -> *mut u8;
}


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
        unsafe { self.allocator.deallocate_range(self.range.clone()) }
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
    /// # Returns
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
    /// # Arguments
    /// + `num`: The number of frames to allocate.
    ///
    /// # Returns
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
    // type Frame: Framesque;

    /// Allocate a new block of size `size` on alignment `align`.
    ///
    /// # Arguments
    /// + `size`: the amount of memory to allocate (in bytes)
    /// + `align`: the alignment for the allocation request
    ///
    /// # Returns
    /// + `Some(*mut u8)` if the request was allocated successfully
    /// + `None` if the allocator is out of memory or if the request was
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
    /// # Arguments
    /// + `frame`: a pointer to the block of memory to deallocate
    /// + `size`: the size of the block being deallocated
    /// + `align`: the alignment of the block being deallocated
    unsafe fn deallocate( &mut self
                        , frame: *mut u8
                        , size: usize, align: usize);

    /// Reallocate `old_frame` from `old_size` bytes to `new_size` bytes
    ///
    /// The `old_size` and `align` parameters _must_ be the same as the
    /// original size and alignment of the frame being reallocated, otherwise
    /// our heap will become corrupted.
    ///
    /// # Arguments
    /// + `old_frame`: a pointer to the frame to be reallocated
    /// + `old_size`: the size (in bytes) of the frame being reallocated
    /// + `new_size`: the size to reallocate the frame to.
    /// + `align`: the alignment for the allocation request
    ///
    /// # Returns
    /// + `Some(*mut u8)` if the frame was reallocated successfully
    /// + `None` if the allocator is out of memory or if the request was
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

#[cfg(feature = "buddy")]
pub mod buddy;
#[cfg(feature = "first_fit")]
pub mod first_fit;
