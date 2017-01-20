//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific paging
use core::mem;

use super::Addr;
use super::paging::Page;

macro_attr! {
    /// A frame (physical page)
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Page!(PAddr) )]
    pub struct PhysicalPage { pub number: u64 }
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

//impl Addr<u64> for PAddr { }
//
//impl_addr! { PAddr, u64 }


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
