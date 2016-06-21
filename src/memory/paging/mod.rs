//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Non-arch-specific paging.
use memory::{Addr, VAddr, PAddr, PAGE_SHIFT};
use memory::alloc::FrameAllocator;

use elf;

use core::{ops, cmp, convert};
use core::ops::Range;

pub use arch::memory::PhysicalPage;
pub use arch::memory::paging::*;

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

pub trait Mapper {
    type Flags;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    /// + `Some(PAddr)` containing the physical address corresponding to
    ///                 `vaddr`, if it is mapped.
    /// + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: VirtualPage) -> Option<PhysicalPage>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>( &mut self, page: VirtualPage, frame: PhysicalPage
             , flags: Self::Flags, alloc: &A )
    where A: FrameAllocator;

    /// Identity map a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>( &mut self, frame: PhysicalPage
                      , flags: Self::Flags, alloc: &A )
    where A: FrameAllocator;

    /// Map the given `VirtualPage` to any free frame.
    ///
    /// This is like the fire and forget version of `map_to`: we just pick the
    /// first available free frame and map the page to it.
    ///
    /// # Arguments
    /// + `page`: the`VirtualPage` to map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to_any<A>( &mut self, page: VirtualPage
                    , flags: Self::Flags
                    , alloc: &A)
    where A: FrameAllocator;

    /// Unmap the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `FrameAllocator`.
    fn unmap<A>(&mut self, page: VirtualPage, alloc: &A)
    where A: FrameAllocator;

}

custom_derive!{
    /// A virtual page
    #[derive( Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Page(VAddr) )]
    pub struct VirtualPage { pub number: usize }
}

impl convert::From<PAddr> for PhysicalPage {
    #[inline] fn from(addr: PAddr) -> Self {
        PhysicalPage::containing(addr)
    }
}

impl convert::From<VAddr> for VirtualPage {
    #[inline] fn from(addr: VAddr) -> Self {
        VirtualPage::containing(addr)
    }
}

impl<'a> convert::From<elf::Section<'a>> for FrameRange {
    #[inline]
    fn from(section: elf::Section<'a>) -> Self {
        let start = PhysicalPage::from(PAddr::from(section.addr()));
        let end = PhysicalPage::from(PAddr::from(section.addr()));
        start .. end
    }

}
