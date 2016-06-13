//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific memory management.
use core::mem;

use ::memory::Addr;
use ::memory::paging::Page;
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
    #[link_name = "heap_base"]
    pub static mut HEAP_BASE: u8;
    #[link_name = "heap_top"]
    pub static mut HEAP_TOP: u8;
    // Of course, we will still need to export the kernel stack addresses like
    // this, but it would be nice if they could be, i dont know, not mut u8s
    // pointers, like God intended.
    #[link_name = "stack_base"]
    pub static mut STACK_BASE: u8;
    #[link_name = "stack_base"]
    pub static mut STACK_TOP: u8;
}

/// A physical (linear) memory address is a 64-bit unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);
derive_addr! { PAddr, u64 }

/// A frame (physical page)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame { pub number: u64 }

impl Page for Frame {
    type Address = PAddr;

    #[inline] fn base(&self) -> Self::Address {
        self.base_addr()
    }

    #[inline] fn containing(addr: Self::Address) -> Self {
        Frame::containing_addr(addr)
    }

    #[inline] fn number(&self) -> usize { self.number as usize }
}

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

impl ops::Sub<usize> for Frame {
    type Output = Frame;

    #[inline]
    fn sub(self, amount: usize) -> Frame {
        Frame { number: self.number - amount as u64 }
    }
}


impl ops::AddAssign for Frame {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.number += rhs.number as u64
    }
}

impl ops::AddAssign<usize> for Frame {
    #[inline(always)]
    fn add_assign(&mut self, rhs: usize) {
        self.number += rhs as u64
    }
}

impl ops::SubAssign for Frame {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.number -= rhs.number as u64
    }
}

impl ops::SubAssign<usize> for Frame {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        self.number -= rhs as u64
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
    pub const fn containing_addr(addr: PAddr) -> Frame {
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

}
