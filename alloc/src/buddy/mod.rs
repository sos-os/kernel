//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Simple buddy-block allocator

#![warn(missing_docs)]
mod math;
#[cfg(feature = "buddy_as_system")]
pub mod system;
#[cfg(feature = "buddy_as_system")]
pub use self::system::BuddyFrameAllocator;

use super::{Allocator, Layout, Address, AllocErr};
use self::math::PowersOf2;

use core::mem;
use core::cmp::{max, min};
use core::ptr::Unique;

use intrusive::list::{List, Node};
use intrusive::rawlink::RawLink;
use memory::PAGE_SIZE;

#[cfg(test)]
mod test;

/// A `FreeList` is a list of unique free blocks
pub type FreeList = List<Unique<FreeBlock>, FreeBlock>;

/// A free block header stores a pointer to the next and previous free blocks.
pub struct FreeBlock { next: RawLink<FreeBlock>
                     , prev: RawLink<FreeBlock>
                     }
impl FreeBlock {
    #[inline] unsafe fn as_ptr(&self) -> *mut u8 { mem::transmute(self) }
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
pub struct Heap<'a> {
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

impl<'a> Heap<'a> {
    /// Construct a new `Heap`.
    ///
    /// # Arguments
    /// + `start_addr`: a pointer to the start location of the heap
    /// + `free_lists`: an array of [`FreeList`]s. The cardinality
    ///    of the `free_lists` array should be equal to the maximum
    ///    allocateable order.
    /// + `heap_size`: the size of the heap (in bytes)
    ///
    /// # Returns
    /// + A new `Heap`, obviously.
    ///
    /// # Panics
    /// + If `start_addr` is a null pointer or is not page-aligned
    /// + If the array of `free_lists` is empty
    /// + If the `heap_size` is too small to contain at least one block, or is
    ///   not a power of two.
    /// + If the calculated minimum block size is to small to contain a
    ///   [`FreeBlock`] header
    ///
    /// # Safety
    /// + If `start_addr` is not valid, you will have a bad time
    ///
    /// [`FreeList`]: type.FreeList.html
    /// [`FreeBlock`]: struct.FreeBlock.html
    pub unsafe fn new( start_addr: *mut u8
                     , free_lists: &'a mut [FreeList]
                     , heap_size: usize)
                     -> Heap<'a> {
        // Cache the number of free lists hopefully saving performance.
        let n_free_lists = free_lists.len();

        assert!( !start_addr.is_null()
                , "Heap start address cannot be null." );
        assert!( n_free_lists > 0
               , "Allocator must have at least one free list.");
        // assert!( start_addr as usize & (PAGE_SIZE-1) as usize == 0
        //        , "Heap start address must be aligned on a 4k boundary.");

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
            = Heap { start_addr: start_addr
                                 , free_lists: free_lists
                                 , heap_size: heap_size
                                 , min_block_size: min_block_size
                                 };

        // the order needed to allocate the entire heap as a single block
        let root_order
            = heap.alloc_order(&Layout::from_size_align(heap_size, 1))
                  .expect("Couldn't determine heap root allocation order!\
                           This should be (as far as I know) impossible.\
                           Something is seriously amiss.");

        // Push the entire heap onto the free lists as the first block.
        heap.push_block(start_addr, root_order);
        heap
    }

    /// Add a block of max order
    ///
    /// # Safety
    /// + This function has no way to guarantee that the given `block` of
    ///   uninitialized memory is not already in use.
    pub unsafe fn add_block(&mut self, block: *mut u8) {
        // TODO: assert the passed block is not a null pointer?
        //       - eliza, 1/23/2017
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
    pub fn alloc_size(&self, layout: &Layout) -> Result<usize, AllocErr> {
        // Pre-check if this is a valid allocation request:
        //  - allocations must be aligned on power of 2 boundaries
        //  - we cannot allocate requests with alignments greater than the
        //    base alignment of the heap without jumping through a bunch of
        //    hoops.
        let align = layout.align();
        debug_assert!(align.is_power_of_two());
        if align > PAGE_SIZE as usize {
            Err(AllocErr::Unsupported {
                details: "Cannot allocate requests with alignments greater \
                          than the base alignment of the heap!"
                })
        // If the request is valid, compute the size we need to allocate
        } else {
            // the allocation size for the request is the next power of 2
            // after the size of the request, the alignment of the request,
            // or the minimum block size (whichever is greatest).
            let alloc_size
                = max!( layout.size(), self.min_block_size, align)
                    .next_power_of_two();

            if alloc_size > self.heap_size {
                // if the calculated size is greater than the size of the heap,
                // we (obviously) cannot allocate this request.
                Err(AllocErr::Unsupported {
                    details: "Cannot allocate requests larger than the heap!"
                })
            } else {
                // otherwise, return the calculated size.
                Ok(alloc_size)
            }
        }
    }

    /// Computes the order of an allocation request.
    ///
    /// The "order" of an allocation refers to the number of times we need to
    /// double the minimum block size to get a large enough block for that
    /// allocation.
    #[inline]
    pub fn alloc_order(&self, layout: &Layout) -> Result<usize, AllocErr> {
        trace!(target: "alloc", "TRACE: alloc_order() called");
        self.alloc_size(layout)
            .map(|s| {
                trace!( target: "alloc"
                      , "in alloc_order(): alloc_size() returned {}", s);
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
    ///
    /// # Arguments
    /// + `block`: a pointer to the block to split
    /// + `old_order`: the current order of `block`
    /// + `new_order`: the requested order for the split block
    ///
    /// # Safety
    /// + This function has no guarantee that `block` is not a null pointer.
    /// + This function has no guarantee that `block` is not already in use --
    ///   if it is invoked on an already allocated block, that block may be
    ///   clobbered without warning.
    /// + This function has no guarantee that `old_order` is the correct order
    ///   for `block`.
    ///
    /// # Panics
    /// + If `new_order` is less than `old_order`: we make a block _larger_ by
    ///   splitting it.
    /// + If `old_order` is larger than the maximum order of this allocator
    unsafe fn split_block( &mut self
                         , block: *mut u8
                         , old_order: usize
                         , new_order: usize ) {
        // TODO: assert the passed block is not a null pointer?
        //       - eliza, 1/23/2017
        trace!( target: "alloc"
              , "split_block() was called, target order: {}.", new_order);

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

            trace!( target: "alloc"
                  , "split block successfully, order: {}, split size: {}"
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


unsafe impl<'a> Allocator for Heap<'a> {

    /// Returns a pointer suitable for holding data described by
    /// `layout`, meeting its size and alignment guarantees.
    ///
    /// The returned block of storage may or may not have its contents
    /// initialized. (Extension subtraits might restrict this
    /// behavior, e.g. to ensure initialization.)
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout`
    /// does not meet allocator's size or alignment constraints.
    ///
    unsafe fn alloc(&mut self, layout: Layout) -> Result<Address, AllocErr> {
        trace!(target: "alloc", "allocate() was called!");
        // First, compute the allocation order for this request
        self.alloc_order(&layout)
            .and_then(|order|
                if order > self.free_lists.len() - 1 {
                    Err(AllocErr::Exhausted { request: layout.clone() })
                } else { Ok(order) } )
            // If the allocation order is defined, then we try to allocate
            // a block of that order. Otherwise, the request is invalid.
            .and_then(|min_order| {
                trace!( target: "alloc"
                      , "in allocate(): min alloc order is {}", min_order);
                // Starting at the minimum possible order...
                // TODO: this is ugly and not FP, rewrite.
                for order in min_order..self.free_lists.len() {
                    if let Some(block) = self.pop_block(order) {
                        trace!(target: "alloc", "in allocate(): found block");
                        if order > min_order {
                            trace!( target: "alloc"
                                  , "in allocate(): order {} is less than \
                                     minimum ({}), splitting."
                                  , order, min_order);
                            self.split_block(block, order, min_order);
                            trace!( target: "alloc"
                                  , "in allocate(): split_block() done");

                        }
                        return Ok(block)
                    }
                }
                Err(AllocErr::Exhausted { request: layout })
            })
    }

    /// Release an allocated block of memory.
    ///
    /// The `size` and `align` parameters _must_ be the same as the original
    /// size and alignment of the frame being deallocated, otherwise our
    /// heap will become corrupted.
    ///
    /// # Arguments
    /// + `frame`: a pointer to the block of memory to deallocate
    /// + `size`: the size of the block being deallocated
    /// + `align`: the alignment of the block being deallocated
    unsafe fn dealloc(&mut self, ptr: Address, layout: Layout) {
        let min_order = self.alloc_order(&layout).unwrap();

        // Check if the deallocated block's buddy block is also free.
        // If it is, merge the two blocks.
        let mut new_block = ptr;
        for order in min_order..self.free_lists.len() {
            // If there is a buddy for this block of the given order...
            if let Some(buddy) = self.get_buddy(order, ptr) {
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
