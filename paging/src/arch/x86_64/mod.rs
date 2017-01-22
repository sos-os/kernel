//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Paging
//!
//! The `x86_64` architecture uses a four-level page table structure. The top
//! page table is called the Page Meta-Level 4 (PML4) table, followed by
//! the Page Directory Pointer Table (PDPT), Page Directory (PD) table, and
//! finally the bottom-level Page Table (PT).
use core::ops;
use core::ptr::Unique;

use alloc::FrameAllocator;
use memory::{PAGE_SIZE, PAddr, Page, PhysicalPage, VAddr, VirtualPage};
use ::Mapper;

use self::table::*;
use self::temp::TempPage;

pub mod table;
pub mod tlb;
pub mod temp;
pub mod cr3;

pub struct ActivePageTable { pml4: ActivePML4 }

impl ops::Deref for ActivePageTable {
    type Target = ActivePML4;

    fn deref(&self) -> &ActivePML4 {
        &self.pml4
    }
}

impl ops::DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut ActivePML4 {
        &mut self.pml4
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable { pml4: ActivePML4::new() }
    }

    /// Execute a closure with the recursive mapping temporarily changed to a
    /// new page table
    pub fn using<F>( &mut self
                   , table: &mut InactivePageTable
                   , temp_page: &mut temp::TempPage
                   , f: F)
    where F: FnOnce(&mut ActivePML4) {
        use self::tlb::flush_all;
        {
            // back up the current PML4 frame
            let prev_pml4_frame = unsafe {
                // this is safe to execute; we are in kernel mode
                cr3::current_pagetable_frame()
            };

            // remap the 511th PML4 entry (the recursive entry) to map to the // frame containing the new PML4.
            self.pml4_mut()[511]
                .set(table.pml4_frame, PRESENT | WRITABLE);
            unsafe {
                // this is safe to execute; we are in kernel mode
                flush_all();
            }

            // execute the closure
            f(self);

            // remap the 511th entry to point back to the original frame
            self.pml4_mut()[511]
                .set(prev_pml4_frame, PRESENT | WRITABLE);

            unsafe {
                // this is safe to execute; we are in kernel mode
                flush_all();
            }
        }
        temp_page.unmap(self);

    }

    /// Replace the current `ActivePageTable` with the given `InactivePageTable`
    ///
    /// # Arguments:
    /// + `new_table`: the `InactivePageTable` that will replace the current
    ///                `ActivePageTable`.
    ///
    /// # Returns:
    /// + the old active page table as an `InactivePageTable`.
    pub fn replace_with(&mut self, new_table: &mut InactivePageTable)
                       -> InactivePageTable {
        unsafe {
            // this is safe to execute; we are in kernel mode
            let old_pml4_frame = cr3::current_pagetable_frame();

            cr3::set_pagetable_frame(new_table.pml4_frame);

            InactivePageTable {
                pml4_frame: old_pml4_frame
            }
        }
    }

}

/// Struct representing the currently active PML4 instance.
///
/// The `ActivePML4` is a `Unique` reference to a PML4-level page table. It's
/// unique because, well, there can only be one active PML4 at a given time.
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
        let addr = page.base();
        let pdpt = self.pml4().next_table(addr);

        let huge_page = || {
            pdpt.and_then(|pdpt|
                pdpt[addr]
                    .do_huge(PDLevel::index_of(addr) + PTLevel::index_of(addr))
                    .or_else(|| {
                        pdpt.next_table(addr).and_then(|pd|
                            pd[addr].do_huge(PTLevel::index_of(addr))
                        )
                    })
                )
        };

        pdpt.and_then(|pdpt| pdpt.next_table(addr))
            .and_then(|pd| pd.next_table(addr))
            .and_then(|pt| pt[addr].get_frame())
            .or_else(huge_page)
    }


    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>( &mut self, page: VirtualPage, frame: PhysicalPage
             , flags: EntryFlags, alloc: &A)
    where A: FrameAllocator {
        // base virtual address of page being mapped
        let addr = page.base();

        // access or create all the lower-level page tables.
        let mut page_table
            // get the PML4
            = self.pml4_mut()
                  // get or create the PDPT table at the page's PML4 index
                  .create_next(addr, alloc)
                  // get or create the PD table at the page's PDPT index
                  .create_next(addr, alloc)
                  // get or create the page table at the  page's PD table index
                  .create_next(addr, alloc);
        trace!(" . . Map: Got page table");
        // check if the page at that index is not currently in use, as we
        // cannot map a page which is currently in use.
        assert!(page_table[addr].is_unused()
               , "Could not map frame {:?}, page table entry {} is already \
                  in use!", frame, PTLevel::index_of(addr));
        // set the page table entry at that index
        page_table[addr].set(frame, flags | table::PRESENT);
    }

    fn identity_map<A>(&mut self, frame: PhysicalPage, flags: EntryFlags
                      , alloc: &A)
    where A: FrameAllocator {
        self.map( Page::containing(VAddr::from(*frame.base_addr() as usize))
                , frame
                , flags
                , alloc )
    }

    fn map_to_any<A>( &mut self
                    , page: VirtualPage
                    , flags: EntryFlags
                    , alloc: &A)
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
    fn unmap<A>(&mut self, page: VirtualPage, alloc: &A)
    where A: FrameAllocator {
        use self::tlb::Flush;

        // get the page table entry corresponding to the page.
        let ref mut entry
            = self.pml4_mut()
                  .page_table_mut_for(page) // get the page table for the page
                  .expect("Could not unmap, huge pages not supported!")
                  [page.base()];        // index the entry from the table

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

    /// Returns true if the given page is mapped.
    #[inline]
    pub fn is_mapped(&self, page: &VirtualPage) -> bool {
         self.translate_page(*page).is_some()
    }


}

/// An inactive page table that the CPU is not currently using
pub struct InactivePageTable {
    pml4_frame: PhysicalPage
}

impl InactivePageTable {
    pub fn new( frame: PhysicalPage
              , active_table: &mut ActivePageTable
              , temp: &mut TempPage)
              -> Self {
        {
            let table = temp.map_to_table(frame.clone(), active_table);
            trace!( " . . . Mapped temp page to table frame.");
            table.zero();
            trace!( " . . . Zeroed inactive table frame.");
            table[511].set( frame.clone(), PRESENT | WRITABLE);
            trace!(" . . . Set active table to point to new inactive table.")
        }
        temp.unmap(active_table);
        trace!(" . . Unmapped temp page.");

        InactivePageTable { pml4_frame: frame }
    }
}

pub fn test_paging<A>(alloc: &A)
where A: FrameAllocator {
    // This testing code shamelessly stolen from Phil Oppermann.
    let mut pml4 = unsafe { ActivePML4::new() };

    // address 0 is mapped
    trace!("Some = {:?}", pml4.translate(VAddr::from(0)));
     // second PT entry
    trace!("Some = {:?}", pml4.translate(VAddr::from(4096)));
    // second PD entry
    trace!("Some = {:?}", pml4.translate(VAddr::from(512 * 4096)));
    // 300th PD entry
    trace!("Some = {:?}", pml4.translate(VAddr::from(300 * 512 * 4096)));
    // second PDPT entry
    trace!("None = {:?}", pml4.translate(VAddr::from(512 * 512 * 4096)));
    // last mapped byte
    trace!("Some = {:?}", pml4.translate(VAddr::from(512 * 512 * 4096 - 1)));


    let addr = VAddr::from(42 * 512 * 512 * 4096); // 42th PDPT entry
    let page = VirtualPage::containing(addr);
    let frame = unsafe { alloc.allocate().expect("no more frames") };
    trace!("None = {:?}, map to {:?}",
             pml4.translate(addr),
             frame);
    pml4.map(page, frame, EntryFlags::empty(), alloc);
    trace!("Some = {:?}", pml4.translate(addr));
    trace!( "next free frame: {:?}"
            , unsafe { alloc.allocate() });

    //trace!("{:#x}", *(Page::containing(addr).as_ptr()));

    pml4.unmap(Page::containing(addr), alloc);
    trace!("None = {:?}", pml4.translate(addr));

}

/// Remaps the kernel using 4KiB pages.
pub fn kernel_remap<A>(info: &multiboot2::Info, alloc: &A) -> ActivePageTable
where A: FrameAllocator {

    // create a  temporary page for switching page tables
    // page number chosen fairly arbitrarily.
    const TEMP_PAGE_NUMBER: usize = 0xcafebabe;
    let mut temp_page = TempPage::new(TEMP_PAGE_NUMBER, alloc);
    trace!(" . . Created temporary page.");

    // old and new page tables
    let mut current_table = unsafe { ActivePageTable::new() };
    trace!(" . . Got current page table.");

    let mut new_table = unsafe {
        InactivePageTable::new(
             alloc.allocate().expect("Out of physical pages!")
          , &mut current_table
          , &mut temp_page
          )
    };
    trace!(" . . Created new page table.");

    // actually remap the kernel --------------------------------------------
    current_table.using(&mut new_table, &mut temp_page, |pml4| {
        let sections // extract allocated ELF sections
            = info.elf_sections()
                  .expect("Can't remap the kernel, no elf sections tag!")
                  .sections()
                  .filter(|section| section.is_allocated());

        for section in sections { // remap ELF secctions
            assert!( section.addr().is_page_aligned()
                   , "Section address must be page aligned to remap!");
            trace!( " . . Identity mapping section at {:?} with size {:?}"
                    , section.addr()
                    , section.length() );

            let flags = EntryFlags::from(&section);

            let start_frame = PhysicalPage::from(section.addr());
            let end_frame = PhysicalPage::from(section.end_addr());

            for frame in start_frame .. end_frame {
                pml4.identity_map(frame, flags, alloc)
            }
        }

        // remap VGA buffer
        trace!( " . . Identity mapping VGA buffer" );
        let vga_buffer_frame = PhysicalPage::from(PAddr::from(0xb8000));
        pml4.identity_map(vga_buffer_frame, WRITABLE, alloc);

        // remap Multiboot info
        trace!( " . . Identity mapping multiboot info" );
        let multiboot_start = PhysicalPage::from(info.start_addr());
        let multiboot_end = PhysicalPage::from(info.end_addr());

        for frame in multiboot_start .. multiboot_end {
            pml4.identity_map(frame, PRESENT, alloc)
        }

    });

    // switch page tables ---------------------------------------------------
    let old_table = current_table.replace_with(&mut new_table);
    trace!(" . . Successfully switched to remapped page table!");

    // create guard page at the location of the old PML4 table
    let old_pml4_page
        = VirtualPage::containing(
            VAddr::from(*(old_table.pml4_frame.base()) as usize)
        );
    current_table.unmap(old_pml4_page, alloc);
    trace!(" . . Unmapped guard page at {:?}", old_pml4_page.base());

    current_table
}
