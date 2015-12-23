//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use ::memory::{PAddr, VAddr, };
use alloc::PAGE_SIZE;
use self::entry::Entry;

pub struct Page { pub number: usize }
pub const N_ENTRIES: usize = 512;

pub type Table = [Entry; N_ENTRIES];

// pub type PML4  = [PML4Entry; 512];
//
// /// A page directory pointer table.
// pub type PDPT  = [PDPTEntry; 512];
//
// /// A page directory.
// pub type PD    = [PDEntry; 512];
//
// /// A page table.
// pub type PT    = [PTEntry; 512];
mod entry {
    pub struct Entry(u64);

    bitflags! {
        flags EntryFlags: u64 {
            const PRESENT =         1 << 0
          , const WRITABLE =        1 << 1
          , const USER_ACCESSIBLE = 1 << 2
          , const WRITE_THROUGH =   1 << 3
          , const NO_CACHE =        1 << 4
          , const ACCESSED =        1 << 5
          , const DIRTY =           1 << 6
          , const HUGE_PAGE =       1 << 7
          , const GLOBAL =          1 << 8
          , const NO_EXECUTE =      1 << 63
        }
    }

    impl Entry {
        #[inline] pub fn is_unused(&self) -> bool {
            self.0 == 0
        }
        #[inline] pub fn flags(&self) -> EntryFlags {
            EntryFlags::from_bits_truncate(self.0)
        }
    }

}
