use arrayvec::ArrayVec;
use memory::{Page, MemRange, PhysicalPage, FrameRange};
use super::FrameAllocator;
use spin::Mutex;

const SIZE: usize = 256;

/// A simple first-fit allocator for allocating page frames.
pub struct FirstFit<'a> {
    frames: &'a Mutex<ArrayVec<[FrameRange; SIZE]>>
}

impl<'a> FrameAllocator for FirstFit<'a> {

    unsafe fn allocate(&self) -> Option<PhysicalPage> {
        unimplemented!()
    }

    unsafe fn deallocate(&self, _frame: PhysicalPage) {
        unimplemented!()
    }

    unsafe fn allocate_range(&self, num: usize) -> Option<FrameRange> {
        let mut frames = self.frames.lock();
        frames.iter()
            .position(|range| range.length() >= num)
            .map(|i| {
                if num < frames[i].length() {
                    frames[i].drop_front(num);
                } else {
                    frames.remove(i);
                }
                frames[i].start.range_of(num)
            })
    }

    unsafe fn deallocate_range(&self, _range: FrameRange) {
        unimplemented!()
    }

}
