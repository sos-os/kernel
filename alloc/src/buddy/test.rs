use super::*;

use ::Allocator;

use core::ptr;

extern "C" {
    /// We need this to allocate aligned memory for our heap.
    fn memalign(alignment: usize, size: usize) -> *mut u8;

    // Release our memory.
    fn free(ptr: *mut u8);
}

const HEAP_ALIGN: usize = 4096;
const HEAP_SIZE: usize = 256;

#[test]
fn test_allocation_size_and_order() {
    unsafe {
        let mem = memalign(HEAP_ALIGN, HEAP_SIZE);
        let mut free_lists: [FreeList; 5]
            = [ FreeList::new(), FreeList::new()
              , FreeList::new(), FreeList::new()
              , FreeList::new()
              ];
        let heap = HeapAllocator::new( mem
                                          , &mut free_lists
                                          , HEAP_SIZE );

        // TEST NEEDED: Can't align beyond MIN_HEAP_ALIGN.

        // Can't align beyond heap_size.
        assert_eq!(None, heap.alloc_size(256, 256*2));

        // Simple allocations just round up to next block size.
        assert_eq!(Some(16), heap.alloc_size(0, 1));
        assert_eq!(Some(16), heap.alloc_size(1, 1));
        assert_eq!(Some(16), heap.alloc_size(16, 1));
        assert_eq!(Some(32), heap.alloc_size(17, 1));
        assert_eq!(Some(32), heap.alloc_size(32, 32));
        assert_eq!(Some(256), heap.alloc_size(256, 256));

        // Aligned allocations use alignment as block size.
        assert_eq!(Some(64), heap.alloc_size(16, 64));

        // Block orders.
        assert_eq!(Some(0), heap.alloc_order(0, 1));
        assert_eq!(Some(0), heap.alloc_order(1, 1));
        assert_eq!(Some(0), heap.alloc_order(16, 16));
        assert_eq!(Some(1), heap.alloc_order(32, 32));
        assert_eq!(Some(2), heap.alloc_order(64, 64));
        assert_eq!(Some(3), heap.alloc_order(128, 128));
        assert_eq!(Some(4), heap.alloc_order(256, 256));
        assert_eq!(None, heap.alloc_order(512, 512));

        free(mem);
    }
}

#[test]
fn test_get_buddy() {
    unsafe {
        let mem = memalign(HEAP_ALIGN, HEAP_SIZE);
        let mut free_lists: [FreeList; 5]
            = [ FreeList::new(), FreeList::new()
              , FreeList::new(), FreeList::new()
              , FreeList::new()
              ];
        let heap = HeapAllocator::new( mem
                                          , &mut free_lists
                                          , HEAP_SIZE );
        let block_16_0 = mem;
        let block_16_1 = mem.offset(16);
        assert_eq!(Some(block_16_1), heap.get_buddy(0, block_16_0));
        assert_eq!(Some(block_16_0), heap.get_buddy(0, block_16_1));

        let block_32_0 = mem;
        let block_32_1 = mem.offset(32);
        assert_eq!(Some(block_32_1), heap.get_buddy(1, block_32_0));
        assert_eq!(Some(block_32_0), heap.get_buddy(1, block_32_1));

        let block_32_2 = mem.offset(64);
        let block_32_3 = mem.offset(96);
        assert_eq!(Some(block_32_3), heap.get_buddy(1, block_32_2));
        assert_eq!(Some(block_32_2), heap.get_buddy(1, block_32_3));

        let block_256_0 = mem;
        assert_eq!(None, heap.get_buddy(4, block_256_0));

        free(mem);
    }
}

#[test]
fn test_alloc_and_dealloc() {
    unsafe {
        let mem = memalign(HEAP_ALIGN, HEAP_SIZE);
        let mut free_lists: [FreeList; 5]
            = [ FreeList::new(), FreeList::new()
              , FreeList::new(), FreeList::new()
              , FreeList::new()
              ];
        let mut heap = HeapAllocator::new( mem
                                              , &mut free_lists
                                              , HEAP_SIZE );


        let block_128_0 = heap.allocate(128, 128);
        assert_eq!(Some(mem.offset(0)), block_128_0);

        heap.deallocate(block_128_0.unwrap(), 128,128);

        let block_16_0 = heap.allocate(8, 8);
        assert_eq!(Some(mem), block_16_0);

        let bigger_than_heap = heap.allocate(4096, HEAP_SIZE);
        assert_eq!(None, bigger_than_heap);

        let bigger_than_free = heap.allocate(HEAP_SIZE, HEAP_SIZE);
        assert_eq!(None, bigger_than_free);

        let block_16_1 = heap.allocate(8, 8);
        assert_eq!(Some(mem.offset(16)), block_16_1);

        let block_16_2 = heap.allocate(8, 8);
        assert_eq!(Some(mem.offset(32)), block_16_2);

        let block_32_2 = heap.allocate(32, 32);
        assert_eq!(Some(mem.offset(64)), block_32_2);

        let block_16_3 = heap.allocate(8, 8);
        assert_eq!(Some(mem.offset(48)), block_16_3);

        // let too_fragmented = heap.allocate(64, 64);
        // assert_eq!(None, too_fragmented);
        heap.deallocate(block_16_0.unwrap(), 8, 8);
        heap.deallocate(block_16_3.unwrap(), 8, 8);
        heap.deallocate(block_16_1.unwrap(), 8, 8);
        heap.deallocate(block_16_2.unwrap(), 8, 8);

        heap.deallocate(block_32_2.unwrap(), 32, 32);

        let block_128_0 = heap.allocate(128, 128);
        assert_eq!(Some(mem.offset(0)), block_128_0);

        let block_128_1 = heap.allocate(128, 128);
        assert_eq!(Some(mem.offset(128)), block_128_1);

        heap.deallocate(block_128_1.unwrap(), 128, 128);
        heap.deallocate(block_128_0.unwrap(), 128, 128);

        // And allocate the whole heap, just to make sure everything
        // got cleaned up correctly.
        let block_256_0 = heap.allocate(256, 256).unwrap();
        assert_eq!(mem.offset(0), block_256_0);

        free(mem);
    }
}
