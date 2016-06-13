use arrayvec::ArrayVec;
use memory::paging::{Page, PageRange};
use super::FrameAllocator;

const SIZE: usize = 256;

/// A simple first-fit allocator for allocating page frames.
pub struct FirstFit<'a, Frame>
where Frame: Page
    , Frame: 'a {
    frames: &'a ArrayVec<[PageRange<Frame>; SIZE]>
}

impl<'a, Frame> FrameAllocator<Frame> for FirstFit<'a, Frame>
where Frame: Page
    , Frame: 'a {

    unsafe fn allocate(&self) -> Option<Frame> {
        unimplemented!()
    }

    unsafe fn deallocate(&self, frame: Frame) {
        unimplemented!()
    }

    unsafe fn allocate_range(&self, num: usize) -> Option<PageRange<Frame>> {
        unimplemented!()
    }

    unsafe fn deallocate_range(&self, range: PageRange<Frame>) {
        unimplemented!()
    }

}
