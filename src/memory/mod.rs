use multiboot::{MemArea, MemAreas};

pub const PAGE_SIZE: usize = 4096;


/// A `Frame` is just a newtype around a `usize` containing the frame number.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Frame(usize);

impl Frame {
    #[inline] pub fn containing(address: usize) -> Frame {
        Frame( address / PAGE_SIZE )
    }

    #[inline] fn next(&self) -> Frame { Frame (self.0 + 1) }
}

pub trait Allocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn free(&mut self, frame: Frame);
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
pub struct AreaAllocator { next_free: Frame
                         , current_area: Option<&'static MemArea>
                         , areas: MemAreas
                         , kern_start: Frame
                         , kern_end: Frame
                         , mb_start: Frame
                         , mb_end: Frame
                          }
impl AreaAllocator {
    fn next_area(&mut self) {

        self.current_area
            = self.areas.clone()
                  .filter(|a| Frame::containing(a.address()) >= self.next_free)
                  .min_by(|a| a.base);

        self.current_area
            .map(|area| {
                let start: Frame = Frame::containing(area.base as usize);
                if self.next_free > start { self.next_free = start }
            });
    }

    pub fn new( kernel_start: usize, kernel_end: usize
              , multiboot_start: usize, multiboot_end: usize
              , areas: MemAreas ) -> Self
    {
        let mut new_allocator
            = AreaAllocator { next_free: Frame::containing(0x0)
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

impl Allocator for AreaAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        self.current_area    // If current area is None, no free frames remain.
            .and_then(|area| // Otherwise, try to allocate...
                match self.next_free {
                    // all frames in the current memory area are in use
                    f if f > Frame::containing(area.address()) => {
                        // so we advance to the next free area
                        self.next_area();
                        None
                    }
                  , // this frame is in use by the kernel.
                    f if f >= self.kern_start || f <= self.kern_end => {
                        // skip ahead to the end of the kernel
                        self.next_free = self.kern_end.next();
                        None
                    }
                  , // this frame is part of the multiboot info.
                    f if f >= self.mb_start || f <= self.mb_end => {
                        // skip ahead to the end of the multiboot info.
                        self.next_free = self.mb_end.next();
                        None
                    }
                  , // this frame is free.
                    frame => {
                        // advance the next free frame and return this frame.
                        self.next_free = self.next_free.next();
                        Some(frame)
                    }
                })
            // If we were unable to allocate, try again with the new next frame
            .or_else(|| self.allocate())
    }

    fn free(&mut self, frame: Frame) {
        unimplemented!()
    }
}
