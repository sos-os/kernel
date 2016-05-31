//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use alloc::{Allocator, PAGE_SIZE};

use super::entry;
use super::entry::Entry;

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

/// The number of entries in a page table.
pub const N_ENTRIES: usize = 512;
/// Size of a page table (in bytes)
pub const PAGE_TABLE_SIZE: usize = N_ENTRIES * PAGE_SIZE;

/// PML4 table
pub const PML4: *mut Table<PML4Level>
    = 0o177777_777_777_777_777_0000 as *mut _;

/// A page table
pub struct Table<L>
where L: TableLevel { /// The entries in the page table.
                      pub entries: [Entry; N_ENTRIES]
                    , _level_marker: PhantomData<L>
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
    type Next: TableLevel;
}
impl Sublevel for PML4Level {
    type Next = PDPTLevel;
}
impl Sublevel for PDPTLevel {
    type Next = PDLevel;
}
impl Sublevel for PDLevel {
    type Next = PTLevel;
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

    /// Returns the address of the next table, or None if none exists.
    fn next_table_addr(&self, index: usize) -> Option<usize> {
        let flags = self[index].flags();
        if flags.contains(entry::PRESENT) && !flags.contains(entry::HUGE_PAGE) {
            Some(((self as *const _ as usize) << 9) | (index << 12))
        } else {
            None
        }
    }

    /// Returns the next table, or `None` if none exists
    pub fn next_table(&self, index: usize) -> Option<&Table<L::Next>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    /// Mutably borrows the next table.
    pub fn next_table_mut(&self, index: usize) -> Option<& mut Table<L::Next>> {
        self.next_table_addr(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }


    /// Returns the next table, creating it if it does not exist.
    pub fn create_next<A>(&mut self, index: usize, alloc: &mut A)
                         -> &mut Table<L::Next>
    where A: Allocator {
        if self.next_table(index).is_none() {
            assert!( !self[index].is_huge()
                   , "Couldn't create next table: huge pages not \
                      currently supported.");

            let frame = unsafe {
                alloc.allocate(PAGE_SIZE, PAGE_SIZE)// I hope that's right
                     .expect("Couldn't create next table: no \
                              frames  available!")
            };

            self[index].set(frame, entry::PRESENT | entry::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
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
