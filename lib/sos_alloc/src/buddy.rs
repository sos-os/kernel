use super::RawLink;

use core::mem;

const MIN_ALIGN: usize = 4096;

pub struct Free { next: RawLink<Free> }

pub struct FreeList<'a> {
    /// A pointer to the head of the free list
    head: Option<&'a mut Free>
  , /// Number of blocks in the free list
    pub length: usize
}

impl<'a> FreeList<'a> {

    /// Create a new empty `FreeList`
    const fn new() -> FreeList<'a> {
        FreeList { head: None, length: 0 }
    }

    /// Push a new block onto the free list
    ///
    /// # Unsafe due to
    ///   - `mem::transmute()`
    ///   - Dereferencing a raw pointer
    unsafe fn push(&mut self, block: *mut u8) {
        let block_ptr = block as *mut Free;
        // be nice if rawlink was kinder to pattern-matching but whatever
        *block_ptr = if let Some(head) = self.head.take() {
            Free { next: RawLink::some(head) }
        } else {
            Free { next: RawLink::none() }
        };
        self.head = Some(mem::transmute(block_ptr));
        self.length += 1;
    }
    /// Pop the head block off of the free list.
    ///
    /// # Returns
    ///   - `Some(*mut u8)` if the free list has blocks left
    ///   - `None` if the free list is empty
    ///
    /// # Unsafe due to
    ///   - `mem::transmute()`
    ///   - Dereferencing a raw pointer
    unsafe fn pop(&mut self) -> Option<*mut u8> {
        self.head.take()
            .map(|head| {
                let popped_block
                    = mem::replace(&mut self.head, head.next.resolve_mut());
                let block_ptr: *mut u8
                    = mem::transmute(popped_block);
                block_ptr
            })
    }

    fn iter<'b>(&'b self) -> FreeListIter<'b> {
        // FreeListIter { current: self.head.map(|c| c.borrow())
        //                             .as_ref()
        //              }
        match self.head {
            Some(ref head) => FreeListIter { current: Some(head) }
          , None           => FreeListIter { current: None }
        }
        // unimplemented!()
    }


    fn iter_mut<'b>(&'b mut self) -> FreeListIterMut<'b> {
        // FreeListIterMut { current: self.head.map(|c| *c ).as_mut() }
        match self.head {
            Some(ref mut head) => FreeListIterMut { current: Some(head) }
          , None               => FreeListIterMut { current: None }
        }
    }
}

struct FreeListIter<'a> {
    current: Option<&'a Free>
}

impl<'a> Iterator for FreeListIter<'a> {
    type Item = &'a Free;

    fn next(&mut self) -> Option<&'a Free> {
        self.current
            .map(|c| {
                self.current = unsafe { c.next.resolve() };
                c
            })
    }
}

struct FreeListIterMut<'a> {
    current: Option<&'a mut Free>
}

impl<'a> Iterator for FreeListIterMut<'a> {
    type Item = &'a mut Free;

    fn next(&mut self) -> Option<&'a mut Free> {
        self.current.take()
            .map(|c| {
                self.current = unsafe { c.next.resolve_mut() };
                c
            })
    }
}

pub struct BuddyHeapAllocator<'a> {
    /// Address of the base of the heap. This must be aligned
    /// on a `MIN_ALIGN` boundary.
    start_addr: *mut u8
  , /// The allocator's free list
    free_lists: &'a mut [FreeList<'a>]
  , /// Number of blocks in the heap (must be a power of 2)
    heap_size: usize
  , /// Minimum block size
    min_block_size: usize
}

impl<'a> BuddyHeapAllocator<'a> {
    pub unsafe fn new( start_addr: *mut u8
                     , free_lists: &'a mut [FreeList<'a>]
                     , heap_size: usize)
                     -> BuddyHeapAllocator<'a>
    {
        let n_free_lists = free_lists.len();

        assert!( !start_addr.is_null()
                , "Heap start address cannot be null." );
        assert!( n_free_lists > 0
               , "Allocator must have at least one free list.");
        assert!( start_addr as usize & (MIN_ALIGN-1) == 0
               , "Heap start address must be aligned on a 4k boundary.");

        let min_block_size = heap_size >> (n_free_lists - 1);

        assert!( heap_size >= min_block_size
               , "Heap must be large enough to contain at least one block.");
        assert!( min_block_size >= mem::size_of::<Free>()
               , "Minimum block size must be large enough to contain \
                  the free block header.");

        let mut heap
            = BuddyHeapAllocator { start_addr: start_addr
                                 , free_lists: free_lists
                                 , heap_size: heap_size
                                 , min_block_size: min_block_size
                                 };
        // TODO: put first head block on appropriately-sized freelist
        heap
    }
}
