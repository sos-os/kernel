//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

//! Kernel memory management.
//!
//! This module contains all of the non-arch-specific paging code, and
//! re-exports memory-related definitions.
#![crate_name = "memory"]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]
#![no_std]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate macro_attr;
// #[macro_use] extern crate newtype_derive;
#[macro_use] extern crate log;

#[macro_use] extern crate util;
// #[cfg(not(test))] #[macro_use] extern crate vga;
// extern crate alloc as liballoc; // TODO: workaround

#[macro_use] pub mod macros;
pub mod arch;

// use alloc::buddy;
// use ::params::InitParams;
use core::{ops, cmp, convert};

pub use arch::PAddr;

pub mod alloc;
pub mod paging;

/// Trait representing an address, whether physical or virtual.
pub trait Addr<R>: ops::Add<Self> + ops::Add<R>
                 + ops::Sub<Self> + ops::Sub<R>
                 + ops::Mul<Self> + ops::Mul<R>
                 + ops::Div<Self> + ops::Div<R>
                 + ops::Shl<Self> + ops::Shl<R>
                 + ops::Shr<Self> + ops::Shr<R>
                 + ops::Deref<Target = R>
                 + convert::From<R> + convert::Into<R>
                 + convert::From<*mut u8> + convert::From<*const u8>
                 + Sized { }

//impl Addr<usize> for VAddr { }

//impl_addr! { VAddr, usize }

macro_attr! {
    /// A virtual address is a machine-sized unsigned integer
    #[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Addr!(usize))]
    pub struct VAddr(usize);
}

impl VAddr {
    /// Convert this virtual address to a pointer
    #[inline] pub fn from_ptr<T>(ptr: *mut T) -> Self { VAddr(ptr as usize) }

    /// Convert a `usize` to a virtual address
    #[inline] pub const fn from_usize(u: usize) -> Self { VAddr(u) }

    /// Convert this virtual address to a `usize`.
    #[inline] pub const fn as_usize(&self) -> usize { self.0 }

    /// Calculate the index in the PML4 table corresponding to this address.
    #[inline] pub fn pml4_index(&self) -> usize {
        *((self >> 39) & 0b111111111 as usize)
    }

    /// Calculate the index in the PDPT table corresponding to this address.
    #[inline] pub fn pdpt_index(&self) -> usize {
        *((self >> 30) & 0b111111111)
    }

    /// Calculate the index in the PD table corresponding to this address.
    #[inline] pub fn pd_index(&self) -> usize {
        *((self >> 21) & 0b111111111)
    }

    /// Calculate the index in the PT table corresponding to this address.
    #[inline] pub fn pt_index(&self) -> usize {
        *((self >> 12) & 0b111111111)
    }
}


//
//impl<A, P> convert::From<A> for P
//where P: Page<Address = A>  {
//
//}

/// Trait for a page. These can be virtual pages or physical frames.
pub trait Page
where Self: Sized
    , Self: ops::AddAssign<usize> + ops::SubAssign<usize>
    , Self: cmp::PartialEq<Self> + cmp::PartialOrd<Self>
    , Self: Copy + Clone {

    /// The type of address used to address this `Page`.
    ///
    /// Typically, this is a `PAddr` or `VAddr` (but it could be a "MAddr")
    /// in schemes where we differentiate between physical and machine
    /// addresses. If we ever have those.
    type Address: Addr<Self::R>;
    /// The numeric type representing `Self::Address`.
    type R;

    /// Returns a new `Page` containing the given `Address`.
    ///
    /// N.B. that since trait functions cannot be `const`, implementors of
    /// this trait may wish to provide implementations of this function
    /// outside of the `impl` block and then wrap them here.
    fn containing(addr: Self::Address) -> Self;

    /// Returns the base `address` where this page starts.
    fn base(&self) -> Self::Address;


    ///// Convert the frame into a raw pointer to the frame's base address
    //#[inline]
    //unsafe fn as_ptr<T>(&self) -> *const T {
    //    mem::transmute(self.base())
    //}
    //
    ///// Convert the frame into a raw mutable pointer to the frame's base address
    //#[inline]
    //unsafe fn as_mut_ptr<T>(&self) -> *mut T {
    //    mem::transmute(self.base())
    //}

    /// Returns a `PageRange` between two pages
    fn range_between(start: Self, end: Self) -> PageRange<Self> {
        PageRange { start: start, end: end }
    }

    /// Returns a `FrameRange` on the frames from this frame until the end frame
    fn range_until(&self, end: Self) -> PageRange<Self> {
        PageRange { start: *self, end: end }
    }

    //fn number(&self) -> R;

}

/// A range of `Page`s.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PageRange<P>
where P: Page { start: P, end: P }

impl<P> PageRange<P>
where P: Page
    , P: Clone {

    /// Returns an iterator over this `PageRange`
    pub fn iter<'a>(&'a self) -> PageRangeIter<'a, P> {
        PageRangeIter { range: self, current: self.start.clone() }
    }
}

/// An iterator over a range of pages
pub struct PageRangeIter<'a, P>
where P: Page
    , P: 'a { range: &'a PageRange<P>, current: P }

impl<'a, P> Iterator for PageRangeIter<'a, P>
where P: Page
    , P: Clone {
    type Item = P;

    ///
    fn next(&mut self) -> Option<P> {
        let end = self.range.end;
      assert!(self.range.start <= end);
      if self.current < end {
          let page = self.current.clone();
          self.current += 1;
          Some(page)
      } else {
          None
      }
  }

}

//macro_rules! make_addr_range {
//    $($name:ident, $addr:ty),+ => {$(
//
//    )+}
//}
