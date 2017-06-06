//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific memory management.
use ::{Addr, Page};

use core::{fmt, ops, mem};

pub const PAGE_SHIFT: u8 = 12;
/// The size of a page (4KiB), in bytes
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT; // 4k
/// The size of a large page (2MiB) in bytes
pub const LARGE_PAGE_SIZE: u64 = 1024 * 1024 * 2;
/// The size of a huge page (2GiB) in bytes
pub const HUGE_PAGE_SIZE: u64 = 1024 * 1024 * 1024;


macro_attr! {
    /// A physical (linear) memory address is a 64-bit unsigned integer
    #[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Addr!(u64, 'P'))]
    #[repr(C)]
    pub struct PAddr(u64);
}

macro_attr! {
    /// A frame (physical page)
    //  TODO: consider renaming this to `Frame` (less typing)?
    //      - eliza, 2/28/2017
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Page!(PAddr) )]
    pub struct PhysicalPage { pub number: u64 }
}
impl fmt::Debug for PhysicalPage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "frame #{} at {:#p}", self.number, self.base_addr())
    }
}

impl ops::Add<usize> for PhysicalPage {
    type Output = Self;

    #[inline] fn add(self, rhs: usize) -> Self {
        PhysicalPage { number: self.number +  rhs as u64 }
    }
}

impl ops::Sub<usize> for PhysicalPage {
    type Output = Self;

    #[inline] fn sub(self, rhs: usize) -> Self {
        PhysicalPage { number: self.number -  rhs as u64 }
    }
}

impl ops::AddAssign<usize> for PhysicalPage {
    #[inline] fn add_assign(&mut self, rhs: usize) {
        self.number += rhs as u64;
    }
}

impl ops::SubAssign<usize> for PhysicalPage {
    #[inline] fn sub_assign(&mut self, rhs: usize) {
        self.number -= rhs as u64;
    }
}

impl PhysicalPage {

    /// Returns the physical address where this frame starts.
    #[inline]
    pub const fn base_addr(&self) -> PAddr {
        PAddr(self.number << PAGE_SHIFT)
    }

    /// Returns a new frame containing `addr`
    #[inline]
    pub const fn containing_addr(addr: PAddr) -> PhysicalPage {
        PhysicalPage { number: addr.0 >> PAGE_SHIFT }
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
