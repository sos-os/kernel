//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Non-arch-specific paging.
use memory::{Addr, VAddr, PAddr, PAGE_SHIFT, PAGE_SIZE };
use core::{ops, cmp};

/// Trait for a page. These can be virtual pages or physical frames.
pub trait Page
where Self: Sized
    , Self: ops::AddAssign<usize> + ops::SubAssign<usize>
    , Self: ops::Add<usize, Output=Self> + ops::Sub<usize>
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
    fn range_of(&self, n: usize) -> PageRange<Self> {
        self.range_until(*self + n)
    }

    /// Returns a `PageRange` on the frames from this frame until the end frame
    #[inline]
    fn range_until(&self, end: Self) -> PageRange<Self> {
        PageRange { start: *self, end: end }
    }

    fn number(&self) -> usize;

}

/// A range of `Page`s.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PageRange<P>
where P: Page { start: P, end: P }

impl<P> PageRange<P>
where P: Page
    , P: Clone {

    pub const fn start(&self) -> P { self.start }

   /// Returns a `PageRange` between two pages
   pub const fn between(start: P, end: P) -> PageRange<P> {
       PageRange { start: start, end: end }
   }

    /// Returns an iterator over this `PageRange`
    pub fn iter<'a>(&'a self) -> PageRangeIter<'a, P> {
        PageRangeIter { range: self, current: self.start.clone() }
    }

    /// Returns the number of `Page`s in this ranage
    #[inline]
    pub fn length(&self) -> usize {
        self.end.number() - self.start.number()
    }

    /// Remove `n` pages from the beginning of this `PageRange`
    pub fn drop_front(&mut self, n: usize) -> &mut Self {
        assert!(n < self.length());
        self.start += n;
        self
    }

    /// Remove `n` pages from the end of this `PageRange`
    pub fn drop_back(&mut self, n: usize) -> &mut Self {
        assert!(n < self.length());
        self.end -= n;
        self
    }

    /// Add `n` pages at the front of this `PageRange`
    pub fn add_front(&mut self, n: usize) -> &mut Self {
        self.start -= n;
        self
    }

    /// Add `n` pages at the back of this `PageRange`
    pub fn add_back(&mut self, n: usize) -> &mut Self {
        self.end += n;
        self
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
    type Frame: Page;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    ///  + `Some(PAddr)` containing the physical address corresponding to
    ///       `vaddr`, if it is mapped.
    ///  + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: VirtualPage) -> Option<Self::Frame>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to<A>( &mut self, page: VirtualPage, frame: Self::Frame
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
    fn map_to_any<A>( &mut self, page: VirtualPage, flags: Self::Flags
                    , alloc: &mut A)
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
pub struct VirtualPage { pub number: usize }

impl ops::Add<usize> for VirtualPage {
    type Output = Self;

    #[inline]
    fn add(self, amount: usize) -> Self {
        VirtualPage { number: self.number + amount }
    }
}

impl ops::Sub<usize> for VirtualPage {
    type Output = Self;

    #[inline]
    fn sub(self, amount: usize) -> Self {
        VirtualPage { number: self.number - amount}
    }
}


impl ops::AddAssign for VirtualPage {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.number += rhs.number
    }
}

impl ops::AddAssign<usize> for VirtualPage {
    #[inline(always)]
    fn add_assign(&mut self, rhs: usize) {
        self.number += rhs
    }
}

impl ops::SubAssign for VirtualPage {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.number -= rhs.number
    }
}

impl ops::SubAssign<usize> for VirtualPage {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        self.number -= rhs
    }
}

impl Page for VirtualPage {
    type Address = VAddr;

    /// Create a new `Page` containing the given virtual address.
    //  TODO: rewrite this as `up`/`down` using the page shift, instead.
    fn containing(addr: VAddr) -> Self {
        assert!( *addr < 0x0000_8000_0000_0000 ||
                 *addr >= 0xffff_8000_0000_0000
               , "invalid address: 0x{:x}", addr );
        VirtualPage { number: *addr / PAGE_SIZE as usize }
    }

    /// Return the start virtual address of this page
    #[inline]
    fn base(&self) -> VAddr {
        VAddr::from(self.number << PAGE_SHIFT)
    }

    #[inline] fn number(&self) -> usize {
        self.number
    }
}

impl VirtualPage {

    /// Flush the page from memory
    //  TODO: this is arch-specific, move it to arch
    pub unsafe fn flush(&self) {
        asm!( "invlpg [$0]"
            :
            : "{rax}"(self.base())
            : "memory"
            : "intel", "volatile")
    }

    // TODO: these are arch-specific, move them to `arch`
    table_idx!{
        pml4_index >> 27
        pdpt_index >> 18
        pd_index >> 9
        pt_index >> 0
    }

}
