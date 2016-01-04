//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use core::fmt;
use core::ops;

use alloc::buddy;
pub use arch::memory::PAddr;

/// A virtual address is a machine-sized unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(usize);

impl VAddr {
    #[inline] pub const fn from_usize(u: usize) -> Self {
        VAddr(u)
    }
    #[inline] pub const fn as_usize(&self) -> usize {
        self.0
    }

    /// Calculate the index in the PML4 table corresponding to this address.
    #[inline] pub fn pml4_index(&self) -> usize {
        (self.as_usize() >> 39) & 0b111111111
    }

    /// Calculate the index in the PDPT table corresponding to this address.
    #[inline] pub fn pdpt_index(&self) -> usize {
        (self.as_usize() >> 30) & 0b111111111
    }

    /// Calculate the index in the PD table corresponding to this address.
    #[inline] pub fn pd_index(&self) -> usize {
        (self.as_usize() >> 21) & 0b111111111
    }

    /// Calculate the index in the PT table corresponding to this address.
    #[inline] pub fn pt_index(&self) -> usize {
        (self.as_usize() >> 12) & 0b111111111
    }
}

impl fmt::Binary for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}


extern {
    static mut HEAP_BASE: u8;
    static mut HEAP_TOP: u8;
}

#[inline] pub fn heap_base_addr() -> usize {
    unsafe { (&mut HEAP_BASE as *mut _) as usize }
}

#[inline] pub fn heap_top_addr() -> usize {
    unsafe { (&mut HEAP_TOP as *mut _) as usize }
}

static mut KERNEL_FREE_LISTS: [buddy::FreeList<'static>; 19]
    // TODO: I really wish there was a less awful way to do this...
    = [ buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      ];

pub unsafe fn init_heap() {
    let heap_base_ptr
        = &mut HEAP_BASE as *mut _;
    let heap_size
        = (&mut HEAP_TOP as *mut _) as usize - heap_base_ptr as usize;
    buddy::system::init_heap( heap_base_ptr
                            , &mut KERNEL_FREE_LISTS
                            , heap_size);
}
