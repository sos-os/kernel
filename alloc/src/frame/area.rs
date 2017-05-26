use super::{Frame, FrameRange, Allocator};
use ::{AllocResult, AllocErr, Layout};
use params::{InitParams, mem};
use memory::{Page, PAGE_SIZE, PAddr};
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
                               , kern_start: Frame
                               , kern_end: Frame
                               , mb_start: Frame
                               , mb_end: Frame
                               }
impl<'a> MemMapAllocator<'a> {
    fn next_area(&mut self) {
        // println!("In next_area");
        self.current_area
            = self.areas
                  .clone()
                  .filter(|a|
                      Frame::containing(a.end_addr) >= self.next_free)
                  .min_by_key(|a| a.start_addr);

        self.current_area
            .map(|area| {
                let start = Frame::containing(area.start_addr);
                if self.next_free > start { self.next_free = start }
            });
    }

    pub fn new( kernel_start: usize, kernel_end: usize
              , multiboot_start: usize, multiboot_end: usize
              , areas: mem::Map )
              -> Self {
        let mut new_allocator = MemMapAllocator {
              next_free: Frame::containing(PAddr::new(0x0))
            , current_area: None
            , areas: areas
            , kern_start: Frame::containing(kernel_start)
            , kern_end: Frame::containing(kernel_end)
            , mb_start: Frame::containing(multiboot_start)
            , mb_end: Frame::containing(multiboot_end)
            };
        new_allocator.next_area();
        new_allocator
    }
}

impl<'a> Allocator for MemMapAllocator<'a> {
    // type Frame = Frame;

    unsafe fn allocate(&mut self) -> AllocResult<Frame> {
        // // println!("In alloc method");
        if let Some(area) = self.current_area {
            match self.next_free {
                // all frames in the current memory area are in use
                f if f > Frame::containing(area.end_addr()) => {
                    // so we advance to the next free area

                    // println!("All frames in current area in use.");
                    self.next_area();
                    // println!("...and returning None");
                }
              , // this frame is in use by the kernel.
                f if f >= self.kern_start || f <= self.kern_end => {
                    // skip ahead to the end of the kernel
                    // println!("In kernel frame, skipping.");
                    self.next_free = self.kern_end.next();
                    // println!("...and returning None");
                }
              , // this frame is part of the multiboot info.
                f if f >= self.mb_start || f <= self.mb_end => {
                    // skip ahead to the end of the multiboot info.
                    // println!("In multiboot frame, skipping...");
                    self.next_free = self.mb_end.next();
                    // println!("...and returning None");
                }
              , // this frame is free.
                frame => {
                    // advance the next free frame and return this frame.
                    // println!("In free frame, advancing...");
                    self.next_free = self.next_free.next();
                    // println!("...and returning {:?}", frame);
                    return Ok(frame)
                }
            };
            self.allocate()
        } else {
            // println!("No free frames remain!");
            Err(AllocErr::Exhausted {
                    layout: Layout::from_size_align( PAGE_SIZE, PAGE_SIZE)
            })
        }
        // self.current_area    // If current area is None, no free frames remain.
        //     .and_then(|area| // Otherwise, try to allocate...
        //         match self.next_free {
        //             // all frames in the current memory area are in use
        //             f if f > Frame::containing(area.address()) => {
        //                 // so we advance to the next free area
        //
        //                 // println!("All frames in current area in use.");
        //                 self.next_area();
        //                 // println!("...and returning None");
        //                 None
        //             }
        //           , // this frame is in use by the kernel.
        //             f if f >= self.kern_start || f <= self.kern_end => {
        //                 // skip ahead to the end of the kernel
        //                 // println!("In kernel frame, skipping.");
        //                 self.next_free = self.kern_end.next();
        //                 // println!("...and returning None");
        //                 None
        //             }
        //           , // this frame is part of the multiboot info.
        //             f if f >= self.mb_start || f <= self.mb_end => {
        //                 // skip ahead to the end of the multiboot info.
        //                 // println!("In multiboot frame, skipping...");
        //                 self.next_free = self.mb_end.next();
        //                 // println!("...and returning None");
        //                 None
        //             }
        //           , // this frame is free.
        //             frame => {
        //                 // advance the next free frame and return this frame.
        //                 // println!("In free frame, advancing...");
        //                 self.next_free = self.next_free.next();
        //                 // println!("...and returning {:?}", frame);
        //                 Some(frame)
        //             }
        //         }
        //     )

    }

    /// Deallocate a frame
    unsafe fn deallocate(&mut self, frame: Frame) {
        // just leak it
    }

    /// Allocate a range of frames
    unsafe fn allocate_range(&mut self, num: usize) -> AllocResult<FrameRange> {
        unimplemented!()
    }
    /// Deallocate a range of frames
    unsafe fn deallocate_range(&mut self, range: FrameRange) {
        //just leak it
    }
}
