//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific memory management.
use core::ptr::Unique;
use core::convert;

use ::memory::{VAddr, Addr};
use ::memory::paging::{Page, Mapper};

use alloc::{Allocator};

pub mod table;
pub mod entry;

use self::table::*;


pub const PAGE_SHIFT: u8 = 12;
/// The size of a page (4096 bytes)
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT; // 4096

extern {
    // It would be really nice if there was a less ugly way of doing this...
    pub static mut HEAP_BASE: u8;
    pub static mut HEAP_TOP: u8;
    pub static mut STACK_BASE: u8;
    pub static mut STACK_TOP: u8;
}

/// A frame (physical page)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame { pub number: u64 }

impl Frame {

    /// Returns the physical address where this frame starts.
    #[inline]
    pub const fn start_addr(&self) -> PAddr {
        PAddr(self.number << PAGE_SHIFT)
    }

    /// Returns a new frame containing `addr`
    #[inline]
    pub const fn containing(addr: PAddr) -> Frame {
        Frame { number: addr.0 / PAGE_SIZE }
    }
}



/// A physical (linear) memory address is a 64-bit unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);

impl Addr<u64> for PAddr { }

impl_addr! { PAddr, u64 }

impl convert::Into<usize> for PAddr {
    #[inline] fn into(self) -> usize { self.0 as usize }
}

//
//impl PAddr {
//    //#[inline] pub fn from_ptr(ptr: *mut u8) -> Self { PAddr(ptr as u64) }
//    #[inline] pub const fn from_u64(u: u64) -> Self { PAddr(u) }
//    #[inline] pub const fn as_u64(&self) -> u64 { self.0 }
//}


pub struct ActivePML4(Unique<Table<PML4Level>>);

impl Mapper for ActivePML4 {
    type Flags = entry::Flags;

    fn translate(&self, vaddr: VAddr) -> Option<PAddr> {
        self.translate_page(Page::containing_addr(vaddr))
            .map(|frame| {
                let offset = vaddr.as_usize() % PAGE_SIZE as usize;
                PAddr::from(frame as u64 + offset as u64)
            })
    }

    fn translate_page(&self, page: Page) -> Option<*mut u8> {
        let pdpt = self.pml4().next_table(page.pml4_index());

        let huge_page = || pdpt.and_then(|pdpt| {
            let pdpt_entry = &pdpt[page.pdpt_index()];

            if pdpt_entry.is_huge() {
                // If the PDPT entry contains the huge page flag, and the
                // entry points to the start frame of a page, then the pointed
                // frame is a 1GB huge page
                pdpt_entry.pointed_frame()
                    .map(|start_frame| {
                        assert!( start_frame as usize % table::N_ENTRIES == 0
                               , "Start frame must be aligned on a \
                                  1GB boundary!");
                        (start_frame as usize + page.pd_index()
                                              + page.pt_index()) as *mut u8
                    })

            } else {
                pdpt.next_table(page.pdpt_index())
                    .and_then(|pd| {
                        let pd_entry = &pd[page.pd_index()];

                        if pd_entry.is_huge() {
                            pd_entry.pointed_frame()
                                .map(|start_frame|{
                                    assert!( (start_frame as usize % table::N_ENTRIES) == 0
                                           , "Start frame must be aligned!");
                                    (start_frame as usize + page.pt_index())
                                        as *mut u8
                                })
                        } else {
                            None
                        }
                    })
            }
        });

        pdpt.and_then(|pdpt| pdpt.next_table(page.pdpt_index()))
            .and_then(|pd| pd.next_table(page.pd_index()))
            .and_then(|pt| pt[page.pt_index()].pointed_frame())
            .or_else(huge_page)

    }

}

impl ActivePML4 {

    pub unsafe fn new() -> Self {
        ActivePML4(Unique::new(PML4))
    }

    fn pml4(&self) -> &Table<PML4Level> {
        unsafe { self.0.get() }
    }

    fn pml4_mut(&mut self) -> &mut Table<PML4Level> {
        unsafe { self.0.get_mut() }
    }

    pub fn identity_map<A: Allocator>( &mut self
                                     , frame: *mut u8
                                     , flags: entry::Flags
                                     , allocator: &mut A )  {
        self.map_to( Page::containing_addr(VAddr::from(frame))
                   , frame
                   , flags
                   , allocator )
    }

    pub fn map<A: Allocator>( &mut self
                            , page: Page
                            , flags: entry::Flags
                            , allocator: &mut A)
    {
        //unsafe {
        //    self.map_to( page
        //               , allocator.allocate(PAGE_SIZE, PAGE_SIZE)
        //                          .expect("Couldn't map, out of frames!")
        //               , flags
        //               , allocator );
        //}
    }

    pub fn map_to<A: Allocator>( &mut self
                               , page: Page
                               , frame: *mut u8
                               , flags: entry::Flags
                               , allocator: &mut A) {
        let mut pdpt = self.pml4_mut()
                           .create_next(page.pml4_index(), allocator);
        let mut pd   = pdpt.create_next(page.pdpt_index(), allocator);
        let mut pt   = pd.create_next(page.pd_index(), allocator);

        let idx = page.pt_index();

        assert!(pt[idx].is_unused()
               , "Could not map frame {:?}, page table entry {} is already \
                  in use!", frame, idx);
        pt[idx].set(frame, flags | entry::PRESENT);
    }

}
