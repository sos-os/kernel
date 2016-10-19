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

#[cfg(feature = "buddy")]
extern crate sos_intrusive as intrusive;
// #[cfg(feature = "multiboot")]
// extern crate sos_multiboot2 as multiboot;
#[cfg(feature = "buddy_as_system")]
extern crate spin;
#[cfg(feature = "trace")] #[macro_use]
extern crate sos_vga;

extern crate memory;

#[cfg(test)] #[macro_use]
extern crate std;

#[cfg(feature = "trace")]
macro_rules! trace {
    ($fmt:expr, $($arg:tt)*) => (log!(level: "TRACE", $fmt, $($arg)* ));
    ($fmt:expr) => (log!(level: "TRACE", $fmt));
}

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($fmt:expr) => ();
    ($fmt:expr, $($arg:tt)*) => ();
}


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

#[cfg(feature = "buddy")]
pub mod buddy;
