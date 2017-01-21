use super::{Page, PhysicalPage, VirtualPage, FrameRange, Mapper};
use ::alloc::{FrameAllocator};
use ::VAddr;

use spin::Mutex;

use core::ops;

use super::ActivePageTable;
use arch::memory::paging::table::{Table, PTLevel};

#[derive(Debug)]
pub struct TempPage { page: VirtualPage
                    , frames: FrameCache
                    }

// Deref conversions for `TempPage` allow us to pass it to functions expecting
// a `VirtualPage`.
impl ops::Deref for TempPage {
    type Target = VirtualPage;
    #[inline] fn deref(&self) -> &VirtualPage { &self.page }
}

impl ops::DerefMut for TempPage {
    #[inline] fn deref_mut(&mut self) -> &mut VirtualPage { &mut self.page }
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

    /// Map the `TempPage` to the given frame in the `ActivePageTable`.
    ///
    /// # Arguments
    /// + `frame`: the `PhysicalPage` to map to
    /// + `table`: the `ActivePageTable`
    ///
    /// # Returns
    /// + The `VAddr` of the mapped page.
    pub fn map_to( &mut self
                 , frame: PhysicalPage
                 , table: &mut ActivePageTable)
                 -> VAddr {
        //assert!( !table.is_mapped(self)
                //, "Cannot map {:?}, as it is already mapped", self);
        use super::table::WRITABLE;
        trace!(" . . TempPage::map_to({:?})", frame);
        table.map(self.page, frame, WRITABLE, &self.frames);
        self.page.base()
    }

    pub fn map_to_table( &mut self
                       , frame: PhysicalPage
                       , table: &mut ActivePageTable)
                       -> &mut Table<PTLevel> {
       unsafe {
           &mut *(self.map_to(frame, table).as_mut_ptr::<Table<PTLevel>>())
       }
   }

    pub fn unmap(&mut self, table: &mut ActivePageTable) {
        assert!( table.is_mapped(self)
                , "Cannot unmap {:?}, as it is not mapped", self);
        table.unmap(self.page, &self.frames);
    }
}

#[derive(Debug)]
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
