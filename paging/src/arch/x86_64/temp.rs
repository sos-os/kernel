use memory::{PAGE_SIZE, Page, PhysicalPage, VAddr, VirtualPage, FrameRange};
use alloc::{AllocResult, AllocErr, Layout, FrameAllocator};

use core::ops;

use super::ActivePageTable;
use super::table::{Table, PTLevel};
use ::{Mapper, MapResult, MapErr};

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
    pub fn new<A>(number: usize, alloc: &mut A) -> Self
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
                 -> MapResult<VAddr> {
        //assert!( !table.is_mapped(self)
                //, "Cannot map {:?}, as it is already mapped", self);
        use super::table::WRITABLE;
        trace!(" . . TempPage::map_to({:?})", frame);
        table.map(self.page, frame, WRITABLE, &mut self.frames)
             .map(|_| { self.page.base() })
    }

    pub fn map_to_table( &mut self
                       , frame: PhysicalPage
                       , table: &mut ActivePageTable)
                       -> MapResult<&mut Table<PTLevel>> {
        self.map_to(frame, table)
            .map(|addr| unsafe {
                &mut *(addr.as_mut_ptr::<Table<PTLevel>>())
            })
   }

    pub fn unmap(&mut self, table: &mut ActivePageTable) -> MapResult<()> {
        trace!("unmapping temp page {:?}", self);
        // assert!( table.is_mapped(self)
        //         , "Cannot unmap {:?}, as it is not mapped", self);
        table.unmap(self.page, &mut self.frames)
             .map(|_| { trace!("temp page unmapped") })

    }
}

#[derive(Debug)]
pub struct FrameCache([Option<PhysicalPage>; 3]);

impl FrameCache {

    pub fn new<A>(alloc: &mut A) -> Self
    where A: FrameAllocator {
        unsafe {
            let frames = [ alloc.allocate().ok()
                         , alloc.allocate().ok()
                         , alloc.allocate().ok() ];
            FrameCache(frames)
        }
    }
}

impl FrameAllocator for FrameCache {

    unsafe fn allocate(&mut self) -> AllocResult<PhysicalPage> {
        self.0.iter_mut()
            .find(    |frame| frame.is_some())
            .and_then(|mut frame| frame.take())
            .map(|frame| { trace!("frameCache: alloced {:?}", &frame); frame})
            .ok_or(AllocErr::Exhausted {
                    request: Layout::from_size_align( PAGE_SIZE as usize
                                                    , PAGE_SIZE as usize)
                })
    }

    unsafe fn deallocate(&mut self, frame: PhysicalPage) {
        self.0.iter_mut()
            .find(    |slot| slot.is_none())
            .and_then(|mut slot| { *slot = Some(frame); Some(()) })
            .expect("FrameCache can only hold three frames!");
    }

    unsafe fn allocate_range(&mut self, _num: usize)
                            -> AllocResult<FrameRange> {
        unimplemented!()
    }

    unsafe fn deallocate_range(&mut self, _range: FrameRange) {
        unimplemented!()
    }

}
