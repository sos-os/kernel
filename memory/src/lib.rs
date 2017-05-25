//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
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
#![feature(asm)]
#![feature(step_trait)]
#![feature(linkage)]
#![no_std]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate macro_attr;

#[macro_use] extern crate util;
// #[cfg(not(test))] #[macro_use] extern crate vga;
// extern crate alloc as liballoc; // TODO: workaround

#[macro_use] pub mod macros;
pub mod arch;

// use alloc::buddy;
// use ::params::InitParams;
use core::{ops, cmp, convert};
use util::Align;

pub use arch::{PAddr, PAGE_SHIFT, PAGE_SIZE};

/// Trait representing an address, whether physical or virtual.
pub trait Addr: ops::Add<Self> + ops::Sub<Self>
              + ops::Mul<Self> + ops::Div<Self>
              + ops::Shl<Self> + ops::Shr<Self>
              + ops::BitOr<Self> + ops::BitAnd<Self>
              + convert::From<*mut u8> + convert::From<*const u8>
              + Sized {
    type Repr: Align;

    fn align_down(&self, align: Self::Repr) -> Self;
    fn align_up(&self, align: Self::Repr) -> Self;

    /// Returns true if this address is aligned on a page boundary.
    fn is_page_aligned(&self) -> bool;
}

//impl Addr<usize> for VAddr { }

//impl_addr! { VAddr, usize }

macro_attr! {
    /// A virtual address is a machine-sized unsigned integer
    #[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Addr!(usize))]
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

use core::ops::Range;

pub use arch::PhysicalPage;

pub type PageRange = Range<VirtualPage>;
pub type FrameRange = Range<PhysicalPage>;

/// Trait for a page. These can be virtual pages or physical frames.
pub trait Page
where Self: Sized
    , Self: ops::AddAssign<usize> + ops::SubAssign<usize>
    , Self: ops::Add<usize, Output=Self> + ops::Sub<usize, Output=Self>
    , Self: cmp::PartialEq<Self> + cmp::PartialOrd<Self>
    , Self: Copy + Clone {

    /// The type of address used to address this `Page`.
    ///
    /// Typically, this is a `PAddr` or `VAddr` (but it could be a "MAddr")
    /// in schemes where we differentiate between physical and machine
    /// addresses. If we ever have those.
    type Address: Addr;

    /// Returns a new `Page` containing the given `Address`.
    ///
    /// N.B. that since trait functions cannot be `const`, implementors of
    /// this trait may wish to provide implementations of this function
    /// outside of the `impl` block and then wrap them here.
    fn containing(addr: Self::Address) -> Self;

    /// Returns the base `Address` where this page starts.
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

    /// Returns a `PageRange` of this `Page` and the next `n` pages.
    #[inline]
    fn range_of(&self, n: usize) -> Range<Self> {
        self.range_until(*self + n)
    }

    /// Returns a `PageRange` on the frames from this frame until the end frame
    #[inline]
    fn range_until(&self, end: Self) -> Range<Self> {
        Range { start: *self, end: end }
    }

    fn number(&self) -> usize;

}


macro_attr!{
    /// A virtual page
    #[derive( Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Page!(VAddr) )]
    pub struct VirtualPage { pub number: usize }
}


//
///// A range of `Page`s.
//#[derive(Copy, Clone, Debug, Eq, PartialEq)]
//pub struct Range<P>
//where P: Page { start: P, end: P }
//
pub trait MemRange {
    /// Returns the number of `Page`s in this ranage
    #[inline]
    fn length(&self) -> usize;

    /// Remove `n` pages from the beginning of this `PageRange`
    fn drop_front(&mut self, n: usize) -> &mut Self;

    /// Remove `n` pages from the end of this `PageRange`
    fn drop_back(&mut self, n: usize) -> &mut Self;

    /// Add `n` pages at the front of this `PageRange`
    fn add_front(&mut self, n: usize) -> &mut Self;

    /// Add `n` pages at the back of this `PageRange`
    fn add_back(&mut self, n: usize) -> &mut Self;
}
    //pub const fn start(&self) -> P { self.start }
   //
   ///// Returns a `PageRange` between two pages
   //pub const fn between(start: P, end: P) -> Range<P> {
   //    Range { start: start, end: end }
   //}
   //
   // /// Returns an iterator over this `PageRange`
   // pub fn iter<'a>(&'a self) -> RangeIter<'a, P> {
   //     RangeIter { range: self, current: self.start.clone() }
   // }

impl<P> MemRange for Range<P>
where P: Page {

    /// Returns the number of `Page`s in this ranage
    #[inline]
    fn length(&self) -> usize {
        self.end.number() - self.start.number()
    }

    /// Remove `n` pages from the beginning of this `PageRange`
    fn drop_front(&mut self, n: usize) -> &mut Self {
        assert!(n < self.length());
        self.start += n;
        self
    }

    /// Remove `n` pages from the end of this `PageRange`
    fn drop_back(&mut self, n: usize) -> &mut Self {
        assert!(n < self.length());
        self.end -= n;
        self
    }

    /// Add `n` pages at the front of this `PageRange`
    fn add_front(&mut self, n: usize) -> &mut Self {
        self.start -= n;
        self
    }

    /// Add `n` pages at the back of this `PageRange`
    fn add_back(&mut self, n: usize) -> &mut Self {
        self.end += n;
        self
    }
}

/// An iterator over a range of pages
pub struct RangeIter<'a, P>
where P: Page
    , P: 'a { range: &'a Range<P>, current: P }

impl<'a, P> Iterator for RangeIter<'a, P>
where P: Page
    , P: Clone {
    type Item = P;

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
