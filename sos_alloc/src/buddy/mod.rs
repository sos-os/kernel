//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Simple buddy-block allocator

mod math;
#[cfg(feature = "buddy_as_system")]
pub mod system;

use super::{ Framesque, Allocator };
use self::math::PowersOf2;

use core::mem;
use core::cmp::{max, min};
use core::ptr::Unique;

use intrusive::list::{List, Node};
use intrusive::rawlink::RawLink;

#[cfg(target-os = "linux")]
#[cfg(test)]
mod test;

/// A `FreeList` is a list of unique free blocks
pub type FreeList = List<Unique<FreeBlock>, FreeBlock>;

/// A free block header stores a pointer to the next and previous free blocks.
pub struct FreeBlock { next: RawLink<FreeBlock>
                     , prev: RawLink<FreeBlock>
                     }

impl Framesque for FreeBlock {
    #[inline] fn as_ptr(&self) -> *mut u8 {
        unsafe { mem::transmute(self) } // HOPEFULLY this is good
    }
}

impl Node for FreeBlock {
    #[inline] fn prev(&self) -> &RawLink<FreeBlock> {
        &self.prev
    }
    #[inline] fn next(&self) -> &RawLink<FreeBlock> {
        &self.next
    }
    #[inline] fn prev_mut(&mut self) -> &mut RawLink<FreeBlock> {
        &mut self.prev
    }
    #[inline] fn next_mut(&mut self) -> &mut RawLink<FreeBlock> {
        &mut self.next
    }
}

// Variadic macro for taking the maximum of n > 2 numbers.
// because I'm lazy.
macro_rules! max {
    ($x:expr) => ($x);
    ($x:expr, $($xs:expr),+) => (max($x, max!($($xs),+)));
}

/// Structure with data for implementing the buddy block allocation strategy.
pub struct BuddyHeapAllocator<'a> {
    /// Address of the base of the heap. This must be aligned
    /// on a `MIN_ALIGN` boundary.
    pub start_addr: *mut u8
  , /// The allocator's free list
    free_lists: &'a mut [FreeList]
  , /// Number of blocks in the heap (must be a power of 2)
    pub heap_size: usize
  , /// Minimum block size
    pub min_block_size: usize
}

impl<'a> BuddyHeapAllocator<'a> {
    /// Construct a new `BuddyHeapAllocator`.
    ///
    /// # Arguments:
    /// + `start_addr`: a pointer to the start location of the heap
    /// + `free_lists`: an array of `FreeList`s. The cardinality
    ///     of the `free_lists` array should be equal to the maximum
    ///     allocateable order.
    /// + `heap_size`: the size of the heap (in bytes)
    ///
    /// # Returns:
    /// + A new `BuddyHeapAllocator`, obviously.
    pub unsafe fn new( start_addr: *mut u8
                     , free_lists: &'a mut [FreeList]
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
        assert!( min_block_size >= mem::size_of::<FreeBlock>()
               , "Minimum block size must be large enough to contain \
                  the free block header.");
        assert!( heap_size.is_pow2()
               , "Heap size must be a power of 2.");

    //    // We must have one free list per possible heap block size.
    //    assert_eq!(min_block_size *
    //               (2u32.pow(free_lists.len() as u32 - 1)) as usize,
    //               heap_size);

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
        heap.push_block(start_addr, root_order);
        heap
    }

    /// Add a block of max order
    pub unsafe fn add_block(&mut self, block: *mut u8) {
        let order = self.free_lists.len() -1;
        self.push_block(block, order);
    }

    /// Computes the size of an allocation request.
    ///
    /// # Arguments
    /// + `size`: A `usize` containing the size of the request
    /// + `align`: A `usize` containing the alignment of the request
    ///
    /// # Returns
    /// + `None` if the request is invalid
    /// + `Some(usize)` containing the size needed if the request is valid
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
            // the allocation size for the request is the next power of 2
            // after the size of the request, the alignment of the request,
            // or the minimum block size (whichever is greatest).
            let alloc_size = max!(size, self.min_block_size, align).next_pow2();

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
                trace!("TRACE in alloc_order(): alloc_size() returned {}", s);
                s.log2() - self.min_block_size.log2()
            })
    }

    /// Computes the size  allocated for a request of the given order.
    #[inline]
    fn order_alloc_size(&self, order: usize) -> usize {
        1 << (self.min_block_size.log2() + order)
    }

    #[inline]
    unsafe fn push_block(&mut self, ptr: *mut u8, order: usize) {
        self.free_lists[order]
            .push_front(Unique::new(ptr as *mut FreeBlock))
    }

    #[inline]
    unsafe fn pop_block(&mut self, order: usize) -> Option<*mut u8>{
        self.free_lists[order]
            .pop_front()
            .map(|block| block.get().as_ptr())
    }


    /// Splits a block
    unsafe fn split_block( &mut self
                         , block: *mut u8
                         , old_order: usize
                         , new_order: usize ) {
        trace!("split_block() was called, target order: {}.", new_order);

        assert!( new_order < old_order
               , "Cannot split a block larger than its current order!");
        assert!( old_order <= self.free_lists.len()
               , "Order is too large for this allocator!");

        let mut split_size = self.order_alloc_size(old_order);
        // let mut curr_order = order;
        for order in (new_order..old_order).rev() {
            split_size >>= 1;

            // let split_size = self.order_alloc_size(order);
            self.push_block(block.offset(split_size as isize), order);

            trace!( "split block successfully, order: {}, split size: {}"
                  , order, split_size );

        }
    }

    /// Finds the buddy block for a given block.
    ///
    /// # Arguments
    /// + `order`: the order of the block to find a buddy for
    /// + `block`: a pointer to the block to find a buddy for
    ///
    /// # Returns
    /// + `Some(*mut u8)` pointing to the buddy block if a buddy was found
    /// + `None` if the block was the size of the entire heap
    pub unsafe fn get_buddy( &self
                           , order: usize
                           , block: *mut u8)
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

    /// Finds and removes the target block from the free list.
    ///
    /// # Arguments
    /// + `order`: the order of the free list to remove the block from_raw
    /// + `block`: a pointer to the block to remove
    ///
    /// # Returns
    /// + `true` if the block was found and removed from the free List
    /// + `false` if the block was not found
    pub fn remove_block(&mut self, order: usize, block: *mut u8) -> bool {
        self.free_lists[order]
            .cursor_mut()
            .find_and_remove(|b| b as *const FreeBlock as *const u8 == block)
            .is_some()
    }
}


impl<'a> Allocator for BuddyHeapAllocator<'a> {

    /// Allocate a new block of size `size` on alignment `align`.
    ///
    /// # Arguments:
    /// + `size`: the amount of memory to allocate (in bytes)
    /// + `align`: the alignment for the allocation request
    ///
    /// # Returns:
    /// + `Some(*mut u8)` if the request was allocated successfully
    /// + `None` if the allocator is out of memory or if the request was
    ///     invalid.
    unsafe fn allocate( &mut self
                      , size: usize
                      , align: usize)
                      -> Option<*mut u8> {
        trace!("allocate() was called!");
        // First, compute the allocation order for this request
        self.alloc_order(size, align)
            .and_then(|order| if order > self.free_lists.len() - 1 { None }
                              else {Some(order)} )
            // If the allocation order is defined, then we try to allocate
            // a block of that order. Otherwise, the request is invalid.
            .and_then(|min_order| {
                trace!("in allocate(): min alloc order is {}", min_order);
                // Starting at the minimum possible order...
                // TODO: this is ugly and not FP, rewrite.
                for order in min_order..self.free_lists.len() {
                    if let Some(block) = self.pop_block(order) {
                        trace!( "in allocate(): found block");
                        if order > min_order {
                            trace!( "in allocate(): order {} is less than \
                                     minimum ({}), splitting."
                                  , order, min_order);
                            self.split_block(block, order, min_order);
                            trace!("in allocate(): split_block() done");

                        }
                        return Some(block)
                    }
                }
                None
            })
    }

    /// Release an allocated block of memory.
    ///
    /// The `size` and `align` parameters _must_ be the same as the original
    /// size and alignment of the frame being deallocated, otherwise our
    /// heap will become corrupted.
    ///
    /// # Arguments:
    /// + `frame`: a pointer to the block of memory to deallocate
    /// + `size`: the size of the block being deallocated
    /// + `align`: the alignment of the block being deallocated
    unsafe fn deallocate( &mut self
                        , block: *mut u8
                        , old_size: usize
                        , align: usize ) {
        let min_order = self.alloc_order(old_size, align)
                            .unwrap();

        // Check if the deallocated block's buddy block is also free.
        // If it is, merge the two blocks.
        let mut new_block = block;
        for order in min_order..self.free_lists.len() {
            // If there is a buddy for this block of the given order...
            if let Some(buddy) = self.get_buddy(order, block) {
                // ...and if the buddy was free...
                if self.remove_block(order, buddy) {
                    // ...merge the buddy with the new block (just use
                    // the lower address), and keep going.
                    new_block = min(new_block, buddy);
                    continue;
                }
            }
            // Otherwise, if we've run out of free buddies, push the new
            // merged block onto the free lsit and return.
            self.push_block(new_block, order);
            return;
        }
    }
}
