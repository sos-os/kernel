//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Non-arch-specific paging.
use memory::{VAddr, PAddr};
use arch::memory::{ PAGE_SHIFT, PAGE_SIZE };


/// Trait for a memory allocator which can allocate memory in terms of frames.
pub trait FrameAllocator<Frame> {

    /// Allocate a new `Frame`
    //  TODO: do we want to be able to request a frame size?
    fn alloc_frame(&mut self) -> Option<Frame>;

    /// Deallocate a given `Frame`.
    fn dealloc_frame(&mut self, frame: Frame);
}


pub trait Mapper {
    type Flags;
    type Frame;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    ///  + `Some(PAddr)` containing the physical address corresponding to
    ///       `vaddr`, if it is mapped.
    ///  + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: Page) -> Option<Self::Frame>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to<A>( &mut self, page: Page, frame: Self::Frame
                , flags: Self::Flags, alloc: &mut A )
    where A: FrameAllocator<Self::Frame>;

    /// Identity map a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>( &mut self, frame: Self::Frame
                      , flags: Self::Flags, alloc: &mut A )
    where A: FrameAllocator<Self::Frame>;

    /// Map the given `page` to any free frame.
    ///
    /// This is like the fire and forget version of `map_to`: we just pick the
    /// first available free frame and map the page to it.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to_any<A>(&mut self, page: Page, flags: Self::Flags, alloc: &mut A)
    where A: FrameAllocator<Self::Frame>;

}


macro_rules! table_idx {
    ( $($name:ident >> $shift:expr)* ) => {$(
        pub fn $name(&self) -> usize {
            (self.number >> $shift) & 0o777
        }
    )*};
}

/// A virtual page
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Page { pub number: usize }

impl Page {
    /// Create a new `Page` containing the given virtual address.
    //  TODO: rewrite this as `up`/`down` using the page shift, instead.
    pub fn containing(addr: VAddr) -> Page {
        assert!( *addr < 0x0000_8000_0000_0000 ||
                 *addr >= 0xffff_8000_0000_0000
               , "invalid address: 0x{:x}", addr );
        Page { number: *addr / PAGE_SIZE as usize }
    }

    /// Return the start virtual address of this page
    #[inline]
    pub fn base_addr(&self) -> VAddr {
        VAddr::from(self.number << PAGE_SHIFT)
    }

    /// Flush the page from memory
    pub unsafe fn flush(&self) {
        asm!( "invlpg [$0]"
            :
            : "{rax}"(self.base_addr())
            : "memory"
            : "intel", "volatile")
    }

    table_idx!{
        pml4_index >> 27
        pdpt_index >> 18
        pd_index >> 9
        pt_index >> 0
    }

    /// Returns a `PageRange`
    pub const fn range_between(start: Page, end: Page) -> PageRange {
        PageRange { start: start, end: end }
    }

    /// Returns a `PageRange` on the pages from this page until the end page
    pub const fn range_until(&self, end: Page) -> PageRange {
        PageRange { start: *self, end: end }
    }

}

/// A range of pages
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PageRange { start: Page, end: Page }

impl PageRange {
    /// Returns an iterator over this `PageRange`
    pub fn iter<'a>(&'a self) -> PageRangeIter<'a> {
        PageRangeIter { range: self, current: self.start.clone() }
    }
}

/// An iterator over a range of pages
pub struct PageRangeIter<'a> { range: &'a PageRange, current: Page }

impl<'a> Iterator for PageRangeIter<'a> {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
      let end = self.range.end.number;
      assert!(self.range.start.number <= end);
      if self.current.number < end {
          let page = self.current.clone();
          self.current.number += 1;
          Some(page)
      } else {
          None
      }
  }

}
