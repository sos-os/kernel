mod math;
#[cfg(feature = "buddy_as_system")]
pub mod system;

use super::{ RawLink, Framesque, Allocator };
use self::math::PowersOf2;

use core::mem;
use core::cmp::{max, min};

pub struct Free { next: RawLink<Free> }

impl Framesque for Free {
    #[inline] fn as_ptr(&self) -> *mut u8 {
        unsafe { mem::transmute(self) } // HOPEFULLY this is good
    }
}

// Variadic macro for taking the maximum of n > 2 numbers.
// because I'm lazy.
macro_rules! max {
    ($x:expr) => ($x);
    ($x:expr, $($xs:expr),+) => (max($x, max!($($xs),+)));
}

pub struct FreeList<'a> {
    /// A pointer to the head of the free list
    head: Option<&'a mut Free>
  , /// Number of blocks in the free list
    pub length: usize
}

impl<'a> FreeList<'a> {

    /// Create a new empty `FreeList`
    ///
    pub const fn new() -> FreeList<'a> {
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
            // if the head block is defined, set that block to point to The
            // head block
            Free { next: RawLink::some(head) }
        } else {
            // if the head block is not defined, set this block to point to
            // an empty block
            Free { next: RawLink::none() }
        };
        // replace the head-block pointer with the pushed block
        self.head = Some(mem::transmute(block_ptr));
        // the list is now one block longer
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

    /// Returns true if this `FreeList` has free blocks remaining
    #[inline] fn has_free_blocks(&self) -> bool { self.head.is_some() }

    /// Attempt to remove a block from the free list.
    ///
    /// This function searches the free list for the specified block, and
    /// removes it if it is found, returning whether or not the block was
    /// removed.
    ///
    /// This is quite slow; with a worst-case time complexity of O(log n),
    /// this function is a major bottleneck in our allocator implementation.
    /// By maintaining sorted free lists, we could perhaps improve performance
    /// somewhat.
    ///
    /// # Returns
    ///   - `true` if the block was removed from the free list
    ///   - `false` if the block was not present in the free list
    unsafe fn remove(&mut self, target_block: *mut u8) -> bool {
        let target_ptr = target_block as *mut Free;
        // loop through the free list's iterator to find the target block.
        // this is gross but hopefully much faster than the much prettier
        // recursive strategy.
        for block in self.iter_mut() {
            let block_ptr: *mut Free = block;
            if block_ptr == target_ptr {
                // if we've found the target block, remove it and break
                *block_ptr = Free { next: block.next.take() };
                return true;
            }
        }
        false
    }

    /// Returns an iterator over the blocks in this free list
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

    /// Returns a mutable iterator over the blocks in this free list.
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
    /// Construct a new `BuddyHeapAllocator`.
    ///
    /// # Arguments:
    ///   - `start_addr`: a pointer to the start location of the heap
    ///   - `free_lists`: an array of `FreeList`s. The cardinality
    ///     of the `free_lists` array should be equal to the maximum
    ///     allocateable order.
    ///   - `heap_size`: the size of the heap (in bytes)
    ///
    /// # Returns:
    ///   - A new `BuddyHeapAllocator`, obviously.
    pub unsafe fn new( start_addr: *mut u8
                     , free_lists: &'a mut [FreeList<'a>]
                     , heap_size: usize)
                     -> BuddyHeapAllocator<'a> {
        // Cache the number of free lists hopefully saving performance.
        let n_free_lists = free_lists.len();

        assert!( !start_addr.is_null()
                , "Heap start address cannot be null." );
        assert!( n_free_lists > 0
               , "Allocator must have at least one free list.");
        assert!( start_addr as usize & (::PAGE_SIZE-1) == 0
               , "Heap start address must be aligned on a 4k boundary.");

        let min_block_size = heap_size >> (n_free_lists - 1);

        assert!( heap_size >= min_block_size
               , "Heap must be large enough to contain at least one block.");
        assert!( min_block_size >= mem::size_of::<Free>()
               , "Minimum block size must be large enough to contain \
                  the free block header.");

        // Zero out the free lists in case we were passed existing data.
        for list in free_lists.iter_mut() {
            *list = FreeList::new();
        }

        let mut heap
            = BuddyHeapAllocator { start_addr: start_addr
                                 , free_lists: free_lists
                                 , heap_size: heap_size
                                 , min_block_size: min_block_size
                                 };

        // the order needed to allocate the entire heap as a single block
        let root_order
            = heap.alloc_order(heap_size, 1)
                  .expect("Couldn't determine heap root allocation order!\
                           This should be (as far as I know) impossible.\
                           Something is seriously amiss.");

        // Push the entire heap onto the free lists as the first block.
        heap.free_lists[root_order].push(start_addr);
        heap
    }

    /// Computes the size of an allocation request.
    ///
    /// # Arguments
    ///   - `size`: A `usize` containing the size of the request
    ///   - `align`: A `usize` containing the alignment of the request
    ///
    /// # Returns
    ///   - `None` if the request is invalid
    ///   - `Some(usize)` containing the size needed if the request is valid
    #[inline]
    pub fn alloc_size(&self, size: usize, align: usize) -> Option<usize> {
        // Pre-check if this is a valid allocation request:
        //  - allocations must be aligned on power of 2 boundaries
        //  - we cannot allocate requests with alignments greater than the
        //    base alignment of the heap without jumping through a bunch of
        //    hoops.
        if !align.is_pow2() || align > ::PAGE_SIZE {
            None
        // If the request is valid, compute the size we need to allocate
        } else {
            let alloc_size
                // the allocation size for the request is the next power of 2
                // after the size of the request, the alignment of the request,
                // or the minimum block size (whichever is greatest).
                = max!( size
                        // we can't allocate less than the minimum block size
                      , self.min_block_size
                        // we can't allocate less than the alignment, either
                      , align )
                    .next_pow2();

            if alloc_size > self.heap_size {
                // if the calculated size is greater than the size of the heap,
                // we (obviously) cannot allocate this request.
                None
            } else {
                // otherwise, return the calculated size.
                Some(alloc_size)
            }
        }
    }

    /// Computes the order of an allocation request.
    ///
    /// The "order" of an allocation refers to the number of times we need to
    /// double the minimum block size to get a large enough block for that
    /// allocation.
    #[inline]
    pub fn alloc_order(&self, size: usize, align: usize) -> Option<usize> {
        trace!("TRACE: alloc_order() called");
        self.alloc_size(size, align)
            .map(|s| {
                trace!("TRACE in alloc_order(): alloc_size() returned {}"
                        , s);
                s.log2() - self.min_block_size.log2()
            })
    }

    /// Computes the size  allocated for a request of the given order.
    #[inline]
    fn order_alloc_size(&self, order: usize) -> usize {
        1 << (self.min_block_size.log2() + order)
    }

    /// Splits a block
    unsafe fn split_block( &mut self, block: *mut u8
                         , order: usize, new_order: usize ) {
        assert!( new_order < order
               , "Cannot split a block larger than its current order!");
        assert!( order <= self.free_lists.len()
               , "Order is too large for this allocator!");

        let mut split_size = self.order_alloc_size(order);
        let mut curr_order = order;

        while curr_order > new_order {
            split_size >>= 1;
            curr_order -= 1;

            self.free_lists[curr_order]
                .push(block.offset(split_size as isize))
        }

    }

    pub unsafe fn get_buddy(&self, order: usize, block: *mut u8)
                            -> Option<*mut u8> {
        // Determine the size of the block allocated for the given order
        let block_size = self.order_alloc_size(order);
        if block_size < self.heap_size {
            // Determine the block's position in the heap.
            let block_pos = (block as usize) - (self.start_addr as usize);
            // Calculate the block's buddy by XORing the block's position
            // in the heap with its size.
            Some(self.start_addr.offset((block_pos ^ block_size) as isize))
        } else {
            // If the block is the size of the entire heap, it (obviously)
            // cannot have a buddy block.
            None
        }
    }
}


impl<'a> Allocator for BuddyHeapAllocator<'a> {
    // type *mut u8 = Free;

    /// Allocate a new block of size `size` on alignment `align`.
    ///
    /// # Arguments:
    ///   - `size`: the amount of memory to allocate (in bytes)
    ///   - `align`: the alignment for the allocation request
    ///
    /// # Returns:
    ///   - `Some(*mut u8)` if the request was allocated successfully
    ///   - `None` if the allocator is out of memory or if the request was
    ///     invalid.
    unsafe fn allocate(&mut self, size: usize, align: usize)
                       -> Option<*mut u8> {
        trace!("allocate() was called!");
        // First, compute the allocation order for this request
        self.alloc_order(size, align)
            // If the allocation order is defined, then we try to allocate
            // a block of that order. Otherwise, the request is invalid.
            .and_then(|min_order| {
                trace!("in allocate(): min alloc order is {}", min_order);
                // Starting at the minimum possible order...
                // TODO: this is ugly and not FP, rewrite.
                let mut result = None;
                for order in min_order..self.free_lists.len() {
                    trace!("in allocate(): current order is {}", order);
                    if let Some(block) = self.free_lists[order].pop() {
                        trace!( "in allocate(): found block");
                        if order > min_order {
                            self.split_block(block, order, min_order);
                        }
                        result = Some(block); break;
                    }
                }
                result
                // self.free_lists[min_order..].iter().enumerate()
                // ...find the first free list that has free blocks left.
                    // .find(|&(_, ref f)| f.has_free_blocks())
                    // .and_then(|(order, f)| unsafe {
                    //     let block = f.pop();
                    //     if order > min_order {
                    //         block.map(|b| {
                    //             self.split_block(b, order, min_order);
                    //             block
                    //         });
                    //     }
                    //     block
                    // })
            })
    }

    /// Release an allocated block of memory.
    ///
    /// The `size` and `align` parameters _must_ be the same as the original
    /// size and alignment of the frame being deallocated, otherwise our
    /// heap will become corrupted.
    ///
    /// # Arguments:
    ///   - `frame`: a pointer to the block of memory to deallocate
    ///   - `size`: the size of the block being deallocated
    ///   - `align`: the alignment of the block being deallocated
    unsafe fn deallocate( &mut self, block: *mut u8
                        , old_size: usize, align: usize ) {
        let min_order = self.alloc_order(old_size, align)
                            .unwrap();

        // Check if the deallocated block's buddy block is also free.
        // If it is, merge the two blocks.
        let mut new_block = block;
        for order in min_order..self.free_lists.len() {
            // If there is a buddy for this block of the given order...
            if let Some(buddy) = self.get_buddy(order, block) {
                // ...and if the buddy was free...
                if self.free_lists[order].remove(buddy) {
                    // ...merge the buddy with the new block (just use
                    // the lower address), and keep going.
                    new_block = min(new_block, buddy);
                    continue;
                }
            }
            // Otherwise, if we've run out of free buddies, push the new
            // merged block onto the free lsit and return.
            self.free_lists[order]
                .push(new_block);
            return;
        }
    }
}
