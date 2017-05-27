//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Stack allocator
use alloc::{AllocResult, AllocErr, FrameAllocator, Layout};
use memory::{PageRange, VAddr};
use ::Mapper;
use arch::ActivePageTable;


use core::ops::Range;

pub type Stack = Range<VAddr>;

pub trait StackAllocator {
    fn allocate<A>( &mut self
                      , page_table: &mut ActivePageTable
                      , frames: &mut A
                      , num_pages: usize) -> AllocResult<Stack>
    where A: FrameAllocator;
}

impl StackAllocator for PageRange {

    fn allocate<A>( &mut self
                      , page_table: &mut ActivePageTable
                      , frames: &mut A
                      , num_pages: usize) -> AllocResult<Stack>
    where A: FrameAllocator {
        use memory::{PAGE_SIZE, Page};
        use arch::table::WRITABLE;
        let exhausted = || {
            AllocErr::Exhausted {
                request: Layout::from_size_align( PAGE_SIZE as usize * num_pages
                                                , PAGE_SIZE as usize)
            }
        };
        if num_pages == 0 {
            Err(AllocErr::Unsupported {
                details: "Why would you try to allocate a zero-page stack?"
            })
        } else {
            // clone a working copy of the stack allocator's page range
            // we will only write it back if we successfully allocate a new
            // stack
            let mut working_pages = self.clone();

            // try to get a guard page
            working_pages.next().ok_or_else(&exhausted)?;

            let start_page = working_pages.next().ok_or_else(&exhausted)?;
            let end_page   = if num_pages == 1 { start_page }
                             else { working_pages.nth(num_pages - 2)
                                                 .ok_or_else(&exhausted)?
                             };

            // successfully allocated! write back the working page range
            *self = working_pages;

            for page in start_page .. end_page {
                page_table.map_to_any(page, WRITABLE, frames);
            }

            let stack_top = end_page.end_address();
            Ok(stack_top .. start_page.base())
        }
    }
}
