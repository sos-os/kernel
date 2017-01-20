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

pub use arch::{PAddr, PAGE_SHIFT};

pub mod alloc;
// pub mod paging;

/// Trait representing an address, whether physical or virtual.
pub trait Addr: ops::Add<Self> + ops::Sub<Self>
              + ops::Mul<Self> + ops::Div<Self>
              + ops::Shl<Self> + ops::Shr<Self>
              + convert::From<*mut u8> + convert::From<*const u8>
              + Sized {
    type Repr;
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


//
//impl<A, P> convert::From<A> for P
//where P: Page<Address = A>  {
//
//}


//macro_rules! make_addr_range {
//    $($name:ident, $addr:ty),+ => {$(
//
//    )+}
//}
