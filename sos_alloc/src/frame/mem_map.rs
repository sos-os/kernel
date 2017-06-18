//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! A simple allocator using the [`InitParams`] memory map.
//!
//! This is basically just a bump pointer allocator for frames; since
//! it doesn't support deallocating frames.
use super::{Frame, FrameRange, Allocator};
use ::{AllocResult, AllocErr, Layout};
use params::{InitParams, mem};
use memory::{Page, PAGE_SIZE, PAddr};

use core::iter::Step;
use core::convert::From;
/// A simple area allocator.
///
/// This is based on the memory area allocation scheme described
/// by Phil Oppermann at [http://os.phil-opp.com/allocating-frames.html].
///
/// This is Not A Good Allocation Scheme, as it does not currently support
/// reallocation of freed frames. The plan is that it will only be used
/// initially, and after we've allocated everything once, we'll switch over
/// to a better allocator.
pub struct MemMapAllocator<'a> { next_free: Frame
                               , current_area: Option<&'a mem::Area>
                               , areas: mem::Map<'a>
                               , kernel_frames: FrameRange
                               , mb_frames: FrameRange
                               }
impl<'a> MemMapAllocator<'a> {
    fn next_area(&mut self) {
        // println!("In next_area");
        self.current_area
            = self.areas.clone()
                  .filter(|a|
                      Frame::containing(a.end_addr) >= self.next_free)
                  .min_by_key(|a| a.start_addr)
                  .map(|area| {
                      let start = Frame::containing(area.start_addr);
                      if self.next_free > start { self.next_free = start };
                      area
                  })
    }

}

impl<'a> From<&'a InitParams> for MemMapAllocator<'a> {
    fn from(params: &'a InitParams) -> Self {
        let mut new_allocator = MemMapAllocator {
              next_free: Frame::containing(PAddr::new(0x12000))
            , current_area: None
            , areas: params.mem_map()
            , kernel_frames: params.kernel_frames()
            // TODO: handle non-multiboot case
            , mb_frames: Frame::containing(params.multiboot_start()) ..
                         Frame::containing(params.multiboot_end()).add_one()
            };
        trace!("creating mem map allocator");
        trace!("kernel frames: {:?}", new_allocator.kernel_frames);
        trace!("multiboot frames: {:?}", new_allocator.mb_frames);
        new_allocator.next_area();
        new_allocator
    }
}

impl<'a> Allocator for MemMapAllocator<'a> {
    // type Frame = Frame;

    unsafe fn allocate(&mut self) -> AllocResult<Frame> {
        // // println!("In alloc method");
        if let Some(area) = self.current_area {
            let frame = Frame { number: self.next_free.number };
            match frame {
                // all frames in the current memory area are in use
                f if f > Frame::containing(area.end_addr) => {
                    // so we advance to the next free area

                    // println!("All frames in current area in use.");
                    self.next_area();
                    // println!("...and returning None");
                }
              , // this frame is in use by the kernel.
                f if f >= self.kernel_frames.start && f <= self.kernel_frames.end => {
                    // skip ahead to the end of the kernel
                    // println!("In kernel frame, skipping.");
                    self.next_free = self.kernel_frames.end.add_one();
                    // println!("...and returning None");
                }
              , // this frame is part of the multiboot info.
                f if f >= self.mb_frames.start && f <= self.mb_frames.end=> {
                    // skip ahead to the end of the multiboot info.
                    // println!("In multiboot frame, skipping...");
                    self.next_free = self.mb_frames.end.add_one();
                    // println!("...and returning None");
                }
              , // this frame is free.
                frame => {
                    // advance the next free frame and return this frame.
                    // println!("In free frame, advancing...");
                    self.next_free = self.next_free.add_one();
                    // println!("...and returning {:?}", frame);
                    trace!("allocated {:?}", frame);
                    return Ok(frame)
                }
            };
            self.allocate()
        } else {
            // println!("No free frames remain!");
            Err(AllocErr::Exhausted {
                    request: Layout::from_size_align( PAGE_SIZE as usize, PAGE_SIZE as usize)
            })
        }
    }

    /// Deallocate a frame
    unsafe fn deallocate(&mut self, _frame: Frame) {
        // just leak it
    }

    /// Allocate a range of frames
    unsafe fn allocate_range(&mut self, _num: usize) -> AllocResult<FrameRange> {
        unimplemented!()
    }
    /// Deallocate a range of frames
    unsafe fn deallocate_range(&mut self, _range: FrameRange) {
        //just leak it
    }
}
