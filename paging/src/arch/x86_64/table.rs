//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use alloc::FrameAllocator;
use ::elf;
use memory::{Addr, PAGE_SIZE, PAddr, Page, PhysicalPage, VAddr, VirtualPage};

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::{convert, fmt, intrinsics};

/// The number of entries in a page table.
pub const N_ENTRIES: usize = 512;
/// Size of a page table (in bytes)
pub const PAGE_TABLE_SIZE: usize = N_ENTRIES * PAGE_SIZE as usize;

/// Base virtual address of the PML4 table
pub const PML4_VADDR: u64 =  0xffffffff_fffff000;

/// A pointer to the PML4 table
pub const PML4_PTR: *mut Table<PML4Level> = PML4_VADDR as *mut _;

/// Mask to apply to a page table entry to isolate the flags
pub const ENTRY_FLAGS_MASK: u64 = (PAGE_SIZE as u64 - 1) as u64;

/// A page table
#[repr(C)]
pub struct Table<L>
where L: TableLevel { /// The entries in the page table.
                      entries: [Entry; N_ENTRIES]
                    , _level_marker: PhantomData<L>
                    }

impl<L:TableLevel> fmt::Debug for Table<L> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {:#p}"
              , unsafe { intrinsics::type_name::<L>() }
              , self)
    }
}
// TODO: this can be moved to x86_all; it's the number of level traits that is
//       x86-64 specific.
//          - eliza, 5/29/2017
pub trait TableLevel {
    /// How much to shift an address by to find its index in this table.
    const ADDR_SHIFT_AMOUNT: usize;
    /// How much to shift a page number by to find its index in this level table
    const PAGE_SHIFT_AMOUNT: usize;
    /// Mask for indices
    const INDEX_MASK: usize = 0o777;

}

pub trait IndexOf<I> {
    fn index_of(i: I) -> usize;
}

impl<T> IndexOf<VAddr> for T
where T: TableLevel {

    /// Returns the index in this table for the given virtual address
    #[inline]
    fn index_of(addr: VAddr) -> usize {
        (addr.as_usize() >> Self::ADDR_SHIFT_AMOUNT) & Self::INDEX_MASK
    }

}

impl<T> IndexOf<VirtualPage> for T
where T: TableLevel {
    /// Returns the index in this table for the given virtual page
    #[inline]
    fn index_of(page: VirtualPage) -> usize {
        (page.number >> Self::PAGE_SHIFT_AMOUNT) & Self::INDEX_MASK
    }
}

impl<T> IndexOf<usize> for T
where T: TableLevel {
    // lol
    // i really hope that the compiler understands that this function does
    // absolutely nothing...
    #[inline(always)]
    fn index_of(i: usize) -> usize { i }

}

pub enum PML4Level {}
pub enum PDPTLevel {}
pub enum PDLevel   {}
pub enum PTLevel   {}

impl TableLevel for PML4Level {
    // TODO: make sure these values are correct!
    //          - eliza, 5/29/2017
    const ADDR_SHIFT_AMOUNT: usize = 39;
    const PAGE_SHIFT_AMOUNT: usize = 27;
}
impl TableLevel for PDPTLevel {
    const ADDR_SHIFT_AMOUNT: usize = 30;
    const PAGE_SHIFT_AMOUNT: usize = 18;
}
impl TableLevel for PDLevel   {
    const ADDR_SHIFT_AMOUNT: usize = 21;
    const PAGE_SHIFT_AMOUNT: usize = 9;
}
impl TableLevel for PTLevel   {
    const ADDR_SHIFT_AMOUNT: usize = 12;
    const PAGE_SHIFT_AMOUNT: usize = 0;
}

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

impl Table<PML4Level> {
    #[inline]
    pub fn page_table_for(&self, page: VirtualPage) -> Option<&Table<PTLevel>> {
        self.next_table(page)
            .and_then(|pdpt| pdpt.next_table(page))
            .and_then(|pd| pd.next_table(page))
    }

    #[inline]
    pub fn page_table_mut_for(&mut self, page: VirtualPage)
                             -> Option<&mut Table<PTLevel>> {
        self.next_table_mut(page)
            .and_then(|pdpt| pdpt.next_table_mut(page))
            .and_then(|pd| pd.next_table_mut(page))
    }
}


impl<L, I> Index<I> for Table<L>
where L: TableLevel
    , L: IndexOf<I> {
    type Output = Entry;

    #[inline] fn index(&self, i: I) -> &Entry {
        &self.entries[L::index_of(i)]
    }
}

impl<L, I> IndexMut<I> for Table<L>
where L: TableLevel
    , L: IndexOf<I> {
    #[inline] fn index_mut(&mut self, i: I) -> &mut Entry {
        &mut self.entries[L::index_of(i)]
    }
}


impl<L: TableLevel> Table<L>  {

    /// Zeroes out the page table by setting all entries "unused"
    pub fn zero(&mut self) {
        trace!("zeroing {:?}", self);
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    /// Return the start physical address of this `Table`
    #[inline]
    pub fn start_paddr(&self) -> PAddr {
        PAddr::from(self as *const Self)
    }

    /// Return the `PhysicalPage` containing this table.
    #[inline]
    pub fn frame(&self) -> PhysicalPage {
        PhysicalPage::containing(self.start_paddr())
    }

}

impl<L: Sublevel> Table<L> {


    /// Returns the address of the next table, or None if none exists.
    #[inline]
    fn next_table_addr(&self, i: usize) -> Option<VAddr> {
        let flags = self[i].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE_PAGE) {
            let table_addr = self as *const _ as usize;
            Some(VAddr::from(table_addr << 9) | (i << 12))
        } else {
            None
        }
    }

    /// Returns the next table, or `None` if none exists
    #[inline]
    pub fn next_table<I>(&self, i: I) -> Option<&Table<L::Next>>
    where L: IndexOf<I> {
        self.next_table_addr(L::index_of(i))
            .map(|table_addr| unsafe { &*(table_addr.as_ptr()) })
    }

    /// Mutably borrows the next table.
    #[inline]
    pub fn next_table_mut<I>(&self, i: I) -> Option<& mut Table<L::Next>>
    where L: IndexOf<I>
        , I: fmt::Debug {
        trace!("{:?}, {:?}", self, i);
        self.next_table_addr(L::index_of(i))
            .map(|table_addr| unsafe { &mut *(table_addr.as_mut_ptr()) })
    }


    /// Returns the next table, creating it if it does not exist.
    pub fn create_next<A>(&mut self, i: VirtualPage, alloc: &mut A)
                         -> &mut Table<L::Next>
    where A: FrameAllocator {
        //println!("in create_next");
        if self.next_table(i).is_none() {
            assert!( !self[i].is_huge()
                   , "Couldn't create next table: huge pages not \
                      currently supported.");
            //print!("allocating...");
            let frame = unsafe {
                alloc.allocate()
                     // TODO: would we rather rewrite this to return
                     // a `Result`? I think so.
                     .expect("Couldn't map page, out of frames!")
            };
            //println!("done.");

            self[i].set(frame, PRESENT | WRITABLE);
            //println!("setted.");
            self.next_table_mut(i).unwrap().zero();
            trace!("zeroed");
        }
        self.next_table_mut(i).unwrap()
    }
}



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

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present { self.insert(PRESENT) }
        else { self.remove(PRESENT) }
        self
    }

    #[inline]
    pub fn set_writable(&mut self, writable: bool) -> &mut Self {
        if writable { self.insert(WRITABLE) }
        else { self.remove(WRITABLE) }
        self
    }

    #[inline]
    pub fn set_executable(&mut self, executable: bool) -> &mut Self {
        if executable { self.remove(NO_EXECUTE) }
        else { self.insert(NO_EXECUTE) }
        self
    }
}

#[derive(Debug)]
pub struct Entry(u64);

impl Entry {

    pub fn new(addr: PAddr) -> Self {
        assert!(addr.is_page_aligned());
        Entry(*addr)
    }

    // TODO: this is one of the worst names I have ever given a thing
    #[inline]
    pub fn do_huge(&self, offset: usize) -> Option<PhysicalPage> {
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
    #[inline(never)]
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

    /// Returns the physical address pointed to by this page table entry
    #[inline]
    pub fn get_addr(&self) -> PAddr {
        PAddr::from(self.0 & PML4_VADDR)
    }

    /// Returns the frame in memory pointed to by this page table entry.
    pub fn get_frame(&self) -> Option<PhysicalPage> {
        if self.flags().is_present() {
            // If the entry is present, mask out bits 12-51 and
            Some(PhysicalPage::containing(self.get_addr()))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: PhysicalPage, flags: EntryFlags) {
        let addr: u64 = frame.base_addr().into();
        assert!(addr & !0x000fffff_fffff000 == 0);
        self.0 = addr | flags.bits();
    }

}

impl<'a> convert::From<&'a elf::Section<u64>> for EntryFlags {
    fn from(section: &'a elf::Section<u64>) -> Self {
        *EntryFlags::empty()
            .set_present(section.is_allocated())
            .set_writable(section.is_writable())
            .set_executable(section.is_executable())
    }
}
