use multiboot::{MemArea, MemAreas};
use super::{Framesque, Allocator};

/// A `Frame` is just a newtype around a `usize` containing the frame number.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct FrameNumber(usize);

impl FrameNumber {
    #[inline] pub fn containing(address: usize) -> FrameNumber {
        FrameNumber( address / ::PAGE_SIZE )
    }

    #[inline] fn next(&self) -> FrameNumber { FrameNumber(self.0 + 1) }
}

impl Framesque for FrameNumber {
    #[inline] fn to_ptr(&self) -> *mut u8 {
        self.0 as *mut u8 // HOPEFULLY this is good
    }
}

/// A simple area allocator.
///
/// This is based on the memory area allocation scheme described
/// by Phil Oppermann at [http://os.phil-opp.com/allocating-frames.html].
///
/// This is Not A Good Allocation Scheme, as it does not currently support
/// reallocation of freed frames. The plan is that it will only be used
/// initially, and after we've allocated everything once, we'll switch over
/// to a better allocator.
pub struct SimpleAreaAllocator { next_free: FrameNumber
                               , current_area: Option<&'static MemArea>
                               , areas: MemAreas
                               , kern_start: FrameNumber
                               , kern_end: FrameNumber
                               , mb_start: FrameNumber
                               , mb_end: FrameNumber
                               }
impl SimpleAreaAllocator {
    fn next_area(&mut self) {
        // println!("In next_area");
        self.current_area
            = self.areas
                  .clone()
                  .filter(|a|
                      FrameNumber::containing(a.address()) >= self.next_free)
                  .min_by(|a| a.base);

        self.current_area
            .map(|area| {
                let start = FrameNumber::containing(area.base as usize);
                if self.next_free > start { self.next_free = start }
            });
    }

    pub fn new( kernel_start: usize, kernel_end: usize
              , multiboot_start: usize, multiboot_end: usize
              , areas: MemAreas ) -> Self
    {
        let mut new_allocator = SimpleAreaAllocator {
              next_free: FrameNumber::containing(0x0)
            , current_area: None
            , areas: areas
            , kern_start: FrameNumber::containing(kernel_start)
            , kern_end: FrameNumber::containing(kernel_end)
            , mb_start: FrameNumber::containing(multiboot_start)
            , mb_end: FrameNumber::containing(multiboot_end)
            };
        new_allocator.next_area();
        new_allocator
    }
}

impl Allocator for SimpleAreaAllocator {
    type Frame = FrameNumber;

    fn allocate(&mut self, size: usize, align: usize) -> Option<Self::Frame> {
        // // println!("In alloc method");
        if let Some(area) = self.current_area {
            match FrameNumber{ number: self.next_free_frame.number } {
                // all frames in the current memory area are in use
                f if f > FrameNumber::containing(area.address()) => {
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
                    return Some(frame)
                }
            };
            self.allocate(size, align)
        } else {
            // println!("No free frames remain!");
            None
        }
        // self.current_area    // If current area is None, no free frames remain.
        //     .and_then(|area| // Otherwise, try to allocate...
        //         match self.next_free {
        //             // all frames in the current memory area are in use
        //             f if f > FrameNumber::containing(area.address()) => {
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

    fn deallocate(&mut self, frame: FrameNumber) {
        unimplemented!()
    }

    fn reallocate( &mut self, frame: FrameNumber
                 , size: usize, align: usize) -> Option<FrameNumber> {
        panic!("The simple allocator doesn't support reallocation \
                (as it is not intended for use as a system allocator).")
    }
}
