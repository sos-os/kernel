//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Implementation of the `RawLink` smart-ish pointer.
//!
//! A `RawLink` is a zero-cost abstraction that allows a raw pointer to be used
//! with an `Option`-esque API.
//!
//! TODO: implement all monadic operations over `Option`-esque types (i.e.
//! `map()`, `and_then()`, etc).

use core::ptr;
use core::fmt;
use core::mem;

/// A `RawLink` provides an `Option`-like interface to a raw pointer.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RawLink<T>(*mut T);

unsafe impl<T> Send for RawLink<T>
where T: 'static
    , T: Send {}

unsafe impl<T> Sync for RawLink<T>
where T: Send
    , T: Sync {}

impl<T> Default for RawLink<T> {
    fn default() -> Self { Self::none() }
}

impl<T> fmt::Display for RawLink<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_null() {
            write!(f, "RawLink::none")
        } else {
            unsafe { write!(f, "RawLink::some({})", *self.0) }
        }

    }
}

impl<T> RawLink<T> {

    /// Equivalent of `Option::None` for a `RawLink`
    ///
    /// # Returns
    ///   - A `RawLink<T>` wrapping a null pointer
    #[inline]
    pub const fn none() -> RawLink<T> { RawLink(ptr::null_mut()) }

    /// Equivalent of `Option::Some` for a `RawLink`
    ///
    /// # Returns
    ///   - A `RawLink<T>` wrapping a pointer to the specified value
    #[inline]
    pub fn some(thing: &mut T) -> RawLink<T> { RawLink(thing) }

    pub const fn from_raw(ptr: *mut T) -> RawLink<T> { RawLink(ptr) }

    /// Resolve the `RawLink` to an `Option`
    ///
    /// # Returns
    ///   - `Some<&'a T>` if the `RawLink` is not a null pointer
    ///   - `None` if the `RawLink` is a null pointer
    ///
    /// # Unsafe due to
    ///   - Returning a reference with an arbitrary lifetime
    ///   - Dereferencing a raw pointer
    #[inline]
    pub unsafe fn resolve<'a>(&self) -> Option<&'a T> {
        self.0.as_ref()
    }

    /// Resolve the `RawLink` to an `Option` on a mutable pointer
    ///
    /// # Returns
    ///   - `Some<&'a mut T>` if the `RawLink` is not a null pointer
    ///   - `None` if the `RawLink` is a null pointer
    ///
    /// # Unsafe due to
    ///   - Returning a reference with an arbitrary lifetime
    ///   - Dereferencing a raw pointer
    #[inline]
    pub unsafe fn resolve_mut<'a>(&self) -> Option<&'a mut T> {
        self.0.as_mut()
    }

    /// Please don't do this.
    ///
    /// # Returns
    ///   - The underlying raw pointer (`*mut T`) behind this `RawLink`
    ///
    /// # Unsafe due to
    ///   - Returning a reference with an arbitrary lifetime
    ///   - Dereferencing a raw pointer
    ///   - Please don't do this
    #[inline]
    pub unsafe fn as_raw(&self) -> *mut T { self.0 }

    #[inline]
    pub fn is_some(&self) -> bool { !self.is_none() }

    #[inline]
    pub fn is_none(&self) -> bool { self.0.is_null() }

    /// Returns the `RawLink` and replaces it with `RawLink::none()`.
    #[inline]
    pub fn take(&mut self) -> Self { mem::replace(self, Self::none()) }

    pub unsafe fn map<U, F: FnOnce(T) -> U>(self, _f: F) -> RawLink<U> {
        unimplemented!()
    }
}
//
// impl<T> ops::Deref for RawLink<T> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target { unsafe { self.resolve().unwrap() } }
// }
