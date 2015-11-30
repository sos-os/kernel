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

#![crate_name = "sos_alloc"]
#![crate_type = "lib"]

// The compiler needs to be instructed that this crate is an allocator in order
// to realize that when this is linked in another allocator like jemalloc
// should not be linked in
#![feature(allocator)]
#![allocator]

// Allocators are not allowed to depend on the standard library which in turn
// requires an allocator in order to avoid circular dependencies. This crate,
// however, can use all of libcore.
#![feature(no_std)]
#![no_std]

#![feature( ptr_as_ref
          , const_fn
          , core_slice_ext )]

pub const PAGE_SIZE: usize = 4096;

/// Trait for something that is like a frame.
///
/// Various allocation strategies use different data structures for
/// representing frames. For example, frames may be stored as frame numbers or
/// as nodes in a linked list. To be `Framesque`, an object need only provide
/// a function to convert the frame data to a pointer to the frame in memory.
pub trait Framesque {
    /// Return a pointer to the frame in memory.
    fn to_ptr(&self) -> *mut u8;
}

/// An `Allocator` implements a particular memory allocation strategy.
///
/// `Allocator`s are generally `struct`s that contain any additional
/// information necessary to enact that allocation strategy. Since
/// different `Allocator`s may use different methods of tracking frames,
/// each `Allocator` implementation can provide its own frame type, as long as
/// that frame type is `frameesque` (i.e., it provides some method to resolve
/// it into a pointer to its memory area).
pub trait Allocator {
    type Frame: Framesque;

    fn allocate(&mut self) -> Option<Self::Frame>;
    fn deallocate(&mut self, frame: Self::Frame);
}


mod rawlink;
pub use self::rawlink::RawLink;

pub mod buddy;

#[test]
fn it_works() {
}
