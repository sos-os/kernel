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

use core::{ops, cmp, convert};

pub use arch::memory::{PAddr, HEAP_BASE, HEAP_TOP, PAGE_SHIFT, PAGE_SIZE};

pub mod paging;
#[macro_use] pub mod macros;

/// A virtual address is a machine-sized unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(usize);

pub trait Addr: ops::Add<Self> + ops::Sub<Self>
              + ops::Mul<Self> + ops::Div<Self>
              + ops::Shl<Self> + ops::Shr<Self>
              + convert::From<*mut u8> + convert::From<*const u8>
              + Sized {
    type Repr;
}

derive_addr! { VAddr, usize }

impl VAddr {
    #[inline] pub fn from_ptr<T>(ptr: *mut T) -> Self { VAddr(ptr as usize) }
    #[inline] pub const fn from_usize(u: usize) -> Self { VAddr(u) }
    #[inline] pub const fn as_usize(&self) -> usize { self.0 }

    /// Calculate the index in the PML4 table corresponding to this address.
    #[inline] pub fn pml4_index(&self) -> usize {
        (self >> 39) & 0b111111111
    }

    /// Calculate the index in the PDPT table corresponding to this address.
    #[inline] pub fn pdpt_index(&self) -> usize {
        (self >> 30) & 0b111111111
    }

    /// Calculate the index in the PD table corresponding to this address.
    #[inline] pub fn pd_index(&self) -> usize {
        (self >> 21) & 0b111111111
    }

    /// Calculate the index in the PT table corresponding to this address.
    #[inline] pub fn pt_index(&self) -> usize {
        (self >> 12) & 0b111111111
    }
}


#[inline] pub fn heap_base_addr() -> usize {
    unsafe { (&mut HEAP_BASE as *mut _) as usize }
}

#[inline] pub fn heap_top_addr() -> usize {
    unsafe { (&mut HEAP_TOP as *mut _) as usize }
}

/// Initialise the kernel heap.
//  TODO: this is the Worst Thing In The Universe. De-stupid-ify it.
pub unsafe fn init_heap<'a>() -> Result<&'a str, &'a str> {
    let heap_base_ptr
        = &mut HEAP_BASE as *mut _;
    let heap_size
        = (&mut HEAP_TOP as *mut _) as usize - heap_base_ptr as usize;
    buddy::system::init_heap(heap_base_ptr, heap_size);
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
