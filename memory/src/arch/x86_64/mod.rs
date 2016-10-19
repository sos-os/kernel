//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific memory management.
use core;
use core::mem;

use ::Addr;
//pub mod table;
//pub mod entry;
pub mod paging;

use core::ops;

pub const PAGE_SHIFT: u8 = 12;
/// The size of a page (4mb)
//  TODO: can we possibly rewrite this so that we can handle pages
//        in excess of 4 megs?
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT; // 4096

extern {
    // TODO: It would be really nice if there was a less ugly way of doing
    // this... (read: after the Revolution when we add memory regions to the
    // heap programmatically.)
    #[link_name = "heap_base_addr"]
    pub static HEAP_BASE: PAddr;
    #[link_name = "heap_top_addr"]
    pub static HEAP_TOP: PAddr;
    // Of course, we will still need to export the kernel stack addresses like
    // this, but it would be nice if they could be, i dont know, not mut u8s
    // pointers, like God intended.
    #[link_name = "stack_base_addr"]
    pub static STACK_BASE: PAddr;
    #[link_name = "stack_top_addr"]
    pub static STACK_TOP: PAddr;
}

custom_derive! {
    /// A physical (linear) memory address is a 64-bit unsigned integer
    #[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Addr(u64))]
    #[repr(C)]
    pub struct PAddr(u64);
}


//impl Addr<u64> for PAddr { }
//
//impl_addr! { PAddr, u64 }

/// A frame (physical page)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame { pub number: u64 }

impl ops::Add<u64> for Frame {
    type Output = Frame;

    #[inline]
    fn add(self, amount: u64) -> Frame {
        Frame { number: self.number + amount }
    }
}

impl ops::Add<usize> for Frame {
    type Output = Frame;

    #[inline]
    fn add(self, amount: usize) -> Frame {
        Frame { number: self.number + amount as u64 }
    }
}

impl Frame {

    /// Returns the physical address where this frame starts.
    #[inline]
    pub const fn base_addr(&self) -> PAddr {
        PAddr(self.number << PAGE_SHIFT)
    }

    /// Returns a new frame containing `addr`
    #[inline]
    pub const fn containing(addr: PAddr) -> Frame {
        Frame { number: addr.0 / PAGE_SIZE }
    }

    /// Convert the frame into a raw pointer to the frame's base address
    #[inline]
    pub unsafe fn as_ptr<T>(&self) -> *const T {
        mem::transmute(self.base_addr())
    }

    /// Convert the frame into a raw mutable pointer to the frame's base address
    #[inline]
    pub unsafe fn as_mut_ptr<T>(&self) -> *mut T {
        *self.base_addr() as *mut u8 as *mut T
    }

    /// Returns a `FrameRange`
    pub const fn range_between(start: Frame, end: Frame) -> FrameRange {
        FrameRange { start: start, end: end }
    }

    /// Returns a `FrameRange` on the frames from this frame until the end frame
    pub const fn range_until(&self, end: Frame) -> FrameRange {
        FrameRange { start: *self, end: end }
    }

}

/// A range of frames
pub struct FrameRange { start: Frame, end: Frame }

impl FrameRange {
    /// Returns an iterator over this `FrameRange`
    pub fn iter<'a>(&'a self) -> FrameRangeIter<'a> {
        FrameRangeIter { range: self, current: self.start.clone() }
    }
}

/// An iterator over a range of frames
pub struct FrameRangeIter<'a> { range: &'a FrameRange, current: Frame }

impl<'a> Iterator for FrameRangeIter<'a> {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
      let end = self.range.end.number;
      assert!(self.range.start.number <= end);
      if self.current.number < end {
          let frame = self.current.clone();
          self.current.number += 1;
          Some(frame)
      } else {
          None
      }
  }

}
