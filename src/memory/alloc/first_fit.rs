use arrayvec::ArrayVec;
use memory::paging::{Page, PageRange};
use super::FrameAllocator;
use spin::Mutex;

const SIZE: usize = 256;

/// A simple first-fit allocator for allocating page frames.
pub struct FirstFit<'a, Frame>
where Frame: Page
    , Frame: 'a {
    frames: &'a Mutex<ArrayVec<[PageRange<Frame>; SIZE]>>
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
        let mut frames = self.frames.lock();
        frames.iter()
            .position(|range| range.length() >= num)
            .map(|i| {
                let mut range = frames[i];
                if num < range.length() {
                    range.drop_front(num);
                } else {
                    frames.remove(i);
                }
                range.start().range_of(num)
            })
    }

    unsafe fn deallocate_range(&self, range: PageRange<Frame>) {
        unimplemented!()
    }

}
