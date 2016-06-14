//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use arch::memory::{Frame, PAddr, PAGE_SIZE};

use memory::paging::Page;
use memory::alloc::FrameAllocator;

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

/// The number of entries in a page table.
pub const N_ENTRIES: usize = 512;
/// Size of a page table (in bytes)
pub const PAGE_TABLE_SIZE: usize = N_ENTRIES * PAGE_SIZE as usize;

/// Base virtual address of the PML4 table
pub const PML4_VADDR: u64 = 0xfffffffffffff000;

/// A pointer to the PML4 table
pub const PML4_PTR: *mut Table<PML4Level> = PML4_VADDR as *mut _;

/// A page table
pub struct Table<L>
where L: TableLevel { /// The entries in the page table.
                      entries: [Entry; N_ENTRIES]
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

//impl Table<PML4Level> {
//    #[inline] fn get_pt
//}

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

    /// Zeroes out the page table by setting all entries "unused"
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
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
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
    where A: FrameAllocator<Frame> {
        if self.next_table(index).is_none() {
            assert!( !self[index].is_huge()
                   , "Couldn't create next table: huge pages not \
                      currently supported.");

            let frame = unsafe {
                alloc.allocate()
                     // TODO: would we rather rewrite this to return
                     // a `Result`? I think so.
                     .expect("Couldn't map page, out of frames!")
            };

            self[index].set(frame, PRESENT | WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}


/// A page table entry.
pub struct Entry(u64);

bitflags! {
    pub flags EntryFlags: u64 {
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

impl EntryFlags {
    /// Returns true if this page is huge
    #[inline]
    pub fn is_huge(&self) -> bool {
        self.contains(HUGE_PAGE)
    }

    /// Returns true if this page is present
    #[inline]
    pub fn is_present(&self) -> bool {
        self.contains(PRESENT)
    }
}

impl Entry {

    // TODO: this is one of the worst names I have ever given a thing
    #[inline]
    pub fn do_huge(&self, offset: usize) -> Option<Frame> {
        if self.is_huge() {
            self.get_frame()
                .map(|start_frame| {
                    assert!( start_frame.number as usize % N_ENTRIES == 0
                           , "Start frame must be aligned on a 1GB boundary!");
                    start_frame + offset
                })
        } else {
            None
        }
    }

    /// Returns true if this is an unused entry
    #[inline]
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Sets this entry to be unused
    #[inline]
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns true if this page is huge
    #[inline]
    pub fn is_huge(&self) -> bool {
        self.flags().is_huge()
    }

    /// Access the entry's bitflags.
    #[inline]
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Returns the frame in memory pointed to by this page table entry.
    pub fn get_frame(&self) -> Option<Frame> {
        if self.flags().is_present() {
            // If the entry is present, mask out bits 12-51 and
            Some(Frame::containing(PAddr::from(self.0 & PML4_VADDR)))
        } else { None }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        let addr: u64 = frame.base_addr().into();
        assert!(addr & !PML4_VADDR == 0);
        self.0 = addr | flags.bits();
    }

}
