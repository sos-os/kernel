//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Kernel memory management.
//!
//! This module contains all of the non-arch-specific paging code, and
//! re-exports memory-related definitions.
use alloc::buddy;

use core::{ops, convert};

pub use arch::memory::{ PAddr
                      , PAGE_SHIFT, PAGE_SIZE
                      , HEAP_BASE, HEAP_TOP
                      };

pub mod alloc;
pub mod paging;
#[macro_use] pub mod macros;

pub trait Addr: ops::Add<Self> + ops::Sub<Self>
              + ops::Mul<Self> + ops::Div<Self>
              + ops::Shl<Self> + ops::Shr<Self>
              + convert::From<*mut u8> + convert::From<*const u8>
              + Sized {
    type Repr;
}

custom_derive! {
    /// A virtual address is a machine-sized unsigned integer
    #[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Addr(usize))]
    pub struct VAddr(usize);
}

impl VAddr {
    #[inline] pub fn from_ptr<T>(ptr: *mut T) -> Self { VAddr(ptr as usize) }
    #[inline] pub const fn from_usize(u: usize) -> Self { VAddr(u) }
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


/// Initialise the kernel heap.
//  TODO: this is the Worst Thing In The Universe. De-stupid-ify it.
pub unsafe fn init_heap<'a>() -> Result<&'a str, &'a str> {
    let heap_base_ptr = HEAP_BASE.as_mut_ptr();
    let heap_size: u64 = (HEAP_TOP - HEAP_BASE).into();
    buddy::system::init_heap(heap_base_ptr, heap_size as usize);
    Ok("[ OKAY ]")
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
