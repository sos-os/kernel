//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # SOS Intrusive Collections
//!
//! _Intrusive_ data structures are data structures whose elements are
//! "aware" of the structures in which they are stored. That is to say
//! that data related to the layout and structure of an intrusive collection
//! is stored by the elements in the collection, rather than internally to
//! them.
//!
//! Intrusive data structures are useful for low-level programming in Rust
//! since they do not explicitly allocate memory. This means that we can use
//! intrusive structures to implement the kernel memory allocator and other
//! kernel subsystems which require structures such as lists prior to
//! the initialization of the kernel heap.
//!
//! This crate currently provides an intrusive linked-list implementation.
//!
//! # Features
//! + `use-std`: use the Rust standard library (`std`), rather than `core`.
#![crate_name = "sos_intrusive"]
#![crate_type = "lib"]
#![feature( const_fn, unique )]
#![cfg_attr(not(feature = "use-std"), no_std )]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![cfg_attr(test, feature(box_syntax))]

pub mod rawlink;
pub use rawlink::RawLink;
pub mod list;
pub use list::List;
pub mod stack;
pub use stack::Stack;

#[cfg(test)]
extern crate std;

use core::ptr::Unique;
pub unsafe trait OwnedRef<T> {
    unsafe fn from_raw(ptr: *mut T) -> Self;
    unsafe fn take(self);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

unsafe impl<T> OwnedRef<T> for Unique<T>  {
    #[inline]
    fn get(&self) -> &T {
        unsafe { self.as_ref() }
    }

    #[inline] fn get_mut(&mut self) -> &mut T {
        unsafe { self.as_mut() }
    }

    #[inline]
    unsafe fn take(self) {}

    unsafe fn from_raw(ptr: *mut T) -> Self {
        Unique::new(ptr)
    }
}

#[cfg(any(test, feature = "use-std"))]
unsafe impl<T> OwnedRef<T> for ::std::boxed::Box<T> {

    fn get(&self) -> &T { &**self }
    fn get_mut(&mut self) -> &mut T { &mut **self }

    #[inline] unsafe fn take(self) {
        ::std::boxed::Box::into_raw(self);
    }

    unsafe fn from_raw(ptr: *mut T) -> Self {
        ::std::boxed::Box::from_raw(ptr)
    }
}
