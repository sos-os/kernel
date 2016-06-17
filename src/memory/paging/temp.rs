use super::{PhysicalPage, VirtualPage, FrameRange};
use memory::alloc::{FrameAllocator, BorrowedFrame};
use spin::Mutex;

pub struct TempPage { page: VirtualPage
                    , frames: FrameCache
                    }

impl TempPage {

    /// Create a new `TempPage`.
    ///
    /// # Arguments
    /// + `number`: the page number for the temporary page
    /// + `alloc`: a `FrameAllocator` for allocating the frames to use
    ///            for the temporary page.
    pub fn new<A>(number: usize, alloc: &A) -> Self
    where A: FrameAllocator {
        TempPage { page: VirtualPage { number: number }
                 , frames: FrameCache::new(alloc)
                 }
    }
}

pub struct FrameCache(Mutex<[Option<PhysicalPage>; 3]>);

impl FrameCache {

    pub fn new<A>(alloc: &A) -> Self
    where A: FrameAllocator {
        unsafe {
            let frames = [ alloc.allocate()
                         , alloc.allocate()
                         , alloc.allocate() ];
            FrameCache(Mutex::new(frames))
        }
    }
}

impl FrameAllocator for FrameCache {

    unsafe fn allocate(&self) -> Option<PhysicalPage> {
        self.0.lock()
            .iter_mut()
            .find(    |frame| frame.is_some())
            .and_then(|mut frame| frame.take())
    }

    unsafe fn deallocate(&self, frame: PhysicalPage) {
        self.0.lock()
            .iter_mut()
            .find(    |slot| slot.is_none())
            .and_then(|mut slot| { *slot = Some(frame); Some(()) })
            .expect("FrameCache can only hold three frames!");
    }

    unsafe fn allocate_range(&self, _num: usize) -> Option<FrameRange> {
        unimplemented!()
    }

    unsafe fn deallocate_range(&self, _range: FrameRange) {
        unimplemented!()
    }

}
