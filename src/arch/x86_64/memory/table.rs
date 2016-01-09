//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::mem;

/// The number of entries in a page table.
pub const N_ENTRIES: usize = 512;

/// PML4 table
pub const PML4: *mut Table<PML4Level>
    = 0o177777_777_777_777_777_0000 as *mut _;

pub struct Table<L>
where L: TableLevel { entries: [Entry; N_ENTRIES]
                    , level_marker: PhantomData<L>
                    }



pub trait TableLevel {}
pub enum PML4Level {}
pub enum PDPTLevel {}
pub enum PDLevel   {}
pub enum PTLevel   {}

impl TableLevel for PML4Level {}
impl TableLevel for PDPTLevel {}
impl TableLevel for PDLevel   {}
impl TableLevel for PTLevel   {}

pub trait Sublevel: TableLevel {
    type NextLevel: TableLevel;
}
impl Sublevel for PML4Level {
    type NextLevel = PDPTLevel;
}
impl Sublevel for PDPTLevel {
    type NextLevel = PDLevel;
}
impl Sublevel for PDLevel {
    type NextLevel = PTLevel;
}

impl<L: TableLevel> Index<usize> for Table<L> {
    type Output = Entry;

    #[inline] fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for Table<L> {
    #[inline] fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L: TableLevel> Table<L>  {

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused()
        }
    }

}

impl<L: Sublevel> Table<L> {

    fn next_table_addr(&self, index: usize) -> Option<usize> {
        let flags = self[index].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
            Some(((self as *const _ as usize) << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut(&self, index: usize)
                         -> Option<& mut Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }
}


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
pub struct Entry(u64);

bitflags! {
    flags EntryFlags: u64 {
        /// Present flag.
        /// Must be 1 to map a 2-MByte page or reference a page table.
        const PRESENT =         1 << 0,
        /// Writable flag.
        /// If 0, writes may not be allowed to the 2-MB region controlled
        /// by this entry
        const WRITABLE =        1 << 1
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
    #[inline] pub fn set_unused(&mut self) {
        self.0 = 0;
    }
    #[inline] pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<*mut u8> {
        unsafe {
            if self.flags().contains(PRESENT) {
                Some(mem::transmute(self.0 & 0x000fffff_fffff000))
            } else { None }
        }
    }

    pub fn set(&mut self, frame: *mut u8, flags: EntryFlags) {
        assert!(frame as u64 & !0x000fffff_fffff000 == 0);
        self.0 = (frame as u64) | flags.bits();
    }

}
