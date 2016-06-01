//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use alloc::buddy;

use core::ops;
use core::convert;

pub use arch::memory::{PAddr, HEAP_BASE, HEAP_TOP};

pub mod paging;
#[macro_use] pub mod macros;

/// A virtual address is a machine-sized unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(usize);

pub trait Addr<R>: ops::Add<Self> + ops::Add<R>
                 + ops::Sub<Self> + ops::Sub<R>
                 + ops::Mul<Self> + ops::Mul<R>
                 + ops::Div<Self> + ops::Div<R>
                 + ops::Shl<Self> + ops::Shl<R>
                 + ops::Shr<Self> + ops::Shr<R>
                 + convert::From<R> + convert::Into<R>
                 + convert::From<*mut u8> + convert::From<*const u8>
                 + Sized { }

impl Addr<usize> for VAddr { }

impl_addr! { VAddr, usize }

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

macro_rules! make_addr_range {
    $($name:ident, $addr:ty),+ => {$(

    )+}
}
