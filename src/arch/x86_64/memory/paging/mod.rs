//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Paging
//!
//! The `x86_64` architecture uses a four-level page table structure. The top
//! page table is called the Page Meta-Level 4 (PML4) table, followed by
//! the Page Directory Pointer Table (PDPT), Page Directory (PD) table, and
//! finally the bottom-level Page Table (PT).
use core::ptr::Unique;
use ::memory::VAddr;
use ::memory::paging::{Page, VirtualPage, Mapper};
use ::memory::alloc::FrameAllocator;

use super::{PhysicalPage, PAddr, PAGE_SIZE};

use self::table::*;

pub mod table;
pub mod tlb;

/// Struct representing the currently active PML4 instance.
///
/// The `ActivePML4` is a `Unique` reference to a PML4-level page table. It's
/// unique because, well, there can only be one active PML4 at a given time.
///
///
pub struct ActivePML4(Unique<Table<PML4Level>>);

/// The active PML4 table is the single point of entry for page mapping.
impl Mapper for ActivePML4 {
    type Flags = EntryFlags;

    fn translate(&self, vaddr: VAddr) -> Option<PAddr> {
        let offset = *vaddr % PAGE_SIZE as usize;
        self.translate_page(Page::containing(vaddr))
            .map(|frame| PAddr::from(frame.number + offset as u64) )
    }

    fn translate_page(&self, page: VirtualPage) -> Option<PhysicalPage> {
        let pdpt = self.pml4().next_table(page.pml4_index());

        pdpt.and_then(|pdpt| pdpt.next_table(page.pdpt_index()))
            .and_then(|pd| pd.next_table(page.pd_index()))
            .and_then(|pt| pt[page.pt_index()].get_frame())
            .or_else( || {
                pdpt.and_then(|pdpt| {
                    let pdpt_entry = &pdpt[page.pdpt_index()];
                    pdpt_entry.do_huge(page.pd_index() + page.pt_index() )
                        .or_else(|| {
                            pdpt.next_table(page.pdpt_index())
                                .and_then(|pd| {
                                    let pd_entry = &pd[page.pd_index()];
                                    pd_entry.do_huge(page.pt_index())
                                })
                        })
                    })
                })
    }


    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>( &mut self, page: VirtualPage, frame: PhysicalPage
             , flags: EntryFlags, alloc: &mut A)
    where A: FrameAllocator {

        // get the page table index of the page to map
        let idx = page.pt_index();

        // access or create all the lower-level page tables.
        let mut page_table
            // get the PML4
            = self.pml4_mut()
                  // get or create the PDPT table at the page's PML4 index
                  .create_next(page.pml4_index(), alloc)
                  // get or create the PD table at the page's PDPT index
                  .create_next(page.pdpt_index(), alloc)
                  // get or create the page table at the  page's PD table index
                  .create_next(idx, alloc);

        // check if the page at that index is not currently in use, as we
        // cannot map a page which is currently in use.
        assert!(page_table[idx].is_unused()
               , "Could not map frame {:?}, page table entry {} is already \
                  in use!", frame, idx);
        // set the page table entry at that index
        page_table[idx].set(frame, flags | table::PRESENT);
    }

    fn identity_map<A>(&mut self, frame: PhysicalPage, flags: EntryFlags
                      , alloc: &mut A)
    where A: FrameAllocator {
        self.map( Page::containing(VAddr::from(frame.base_addr().0 as usize))
                , frame
                , flags
                , alloc )
    }

    fn map_to_any<A>( &mut self
                    , page: VirtualPage
                    , flags: EntryFlags
                    , alloc: &mut A)
    where A: FrameAllocator {
        let frame = unsafe {
            alloc.allocate()
             // TODO: would we rather rewrite this to return
             // a `Result`? I think so.
                 .expect("Couldn't map page, out of frames!")
        };
        self.map(page, frame, flags, alloc);
    }

    /// Unmap the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `FrameAllocator`.
    fn unmap<A>(&mut self, page: VirtualPage, alloc: &mut A)
    where A: FrameAllocator {
        use self::tlb::Flush;

        // get the page table entry corresponding to the page.
        let ref mut entry
            = self.pml4_mut()
                  .page_table_mut_for(page) // get the page table for the page
                  .expect("Could not unmap, huge pages not supported!")
                  [page.pt_index()];        // index the entry from the table

        // get the pointed frame for the page table entry.
        let frame = entry.get_frame()
                         .expect("Could not unmap page that was not mapped!");

        // mark the page table entry as unused
        entry.set_unused();

        // deallocate the frame and flush the translation lookaside buffer
        // this is safe because we're in kernel mode
        assert!( page.flush()
               , "Could not flush TLB, we were not in kernel mode!");
        unsafe {
            // this is hopefully safe because nobody else should be using an
            // allocated page frame
            alloc.deallocate(frame);
        }
        // TODO: check if page tables containing the unmapped page are empty
        //       and deallocate them too?
    }

}

impl ActivePML4 {

    pub unsafe fn new() -> Self {
        ActivePML4(Unique::new(PML4_PTR))
    }

    fn pml4(&self) -> &Table<PML4Level> {
        unsafe { self.0.get() }
    }

    fn pml4_mut(&mut self) -> &mut Table<PML4Level> {
        unsafe { self.0.get_mut() }
    }

}

/// An inactive page table that the CPU is not currently using
pub struct InactivePML4 {
    pml4: Table<PML4Level>
}

pub fn test_paging<A>(alloc: &mut A)
where A: FrameAllocator {
    // This testing code shamelessly stolen from Phil Oppermann.
    let mut pml4 = unsafe { ActivePML4::new() };

    // address 0 is mapped
    println!("Some = {:?}", pml4.translate(VAddr::from(0)));
     // second PT entry
    println!("Some = {:?}", pml4.translate(VAddr::from(4096)));
    // second PD entry
    println!("Some = {:?}", pml4.translate(VAddr::from(512 * 4096)));
    // 300th PD entry
    println!("Some = {:?}", pml4.translate(VAddr::from(300 * 512 * 4096)));
    // second PDPT entry
    println!("None = {:?}", pml4.translate(VAddr::from(512 * 512 * 4096)));
    // last mapped byte
    println!("Some = {:?}", pml4.translate(VAddr::from(512 * 512 * 4096 - 1)));


    let addr = VAddr::from(42 * 512 * 512 * 4096); // 42th PDPT entry
    let page = VirtualPage::containing(addr);
    let frame = unsafe { alloc.allocate().expect("no more frames") };
    println!("None = {:?}, map to {:?}",
             pml4.translate(addr),
             frame);
    pml4.map(page, frame, EntryFlags::empty(), alloc);
    println!("Some = {:?}", pml4.translate(addr));
    println!( "next free frame: {:?}"
            , unsafe { alloc.allocate() });

    //println!("{:#x}", *(Page::containing(addr).as_ptr()));

    pml4.unmap(Page::containing(addr), alloc);
    println!("None = {:?}", pml4.translate(addr));

}
