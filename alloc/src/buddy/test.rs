use super::*;

use ::{Allocator, Layout};

use core::ptr;

extern "C" {
    /// We need this to allocate aligned memory for our heap.
    #[cfg(target_os = "macos")]
    #[link_name = "je_posix_memalign"]
    fn memalign(alignment: usize, size: usize) -> *mut u8;

    #[cfg(not(target_os = "macos"))]
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
        let heap = Heap::new( mem, &mut free_lists, HEAP_SIZE );

        // TEST NEEDED: Can't align beyond MIN_HEAP_ALIGN.

        // Can't align beyond heap_size.
        assert!(heap.alloc_size(&Layout::from_size_align(256, 256*2)).is_err());

        macro_rules! assert_size {
            ($(size: $size: expr, align: $align:expr, $result:expr),*) => {
                $(assert_eq!( $result
                            , heap.alloc_size(&Layout::from_size_align($size,
                                $align)));
                 )*
            }
        }

        macro_rules! assert_order {
            ($(size: $size: expr, align: $align:expr,  $result:expr),*) => {
                $(assert_eq!( $result
                            , heap.alloc_order(&Layout::from_size_align($size,
                                $align)));
                 )*
            }
        }

        // Simple allocations just round up to next block size.
        assert_size!{ size: 0, align: 1, Ok(16)
                    , size: 1, align: 1, Ok(16)
                    , size: 16, align: 1, Ok(16)
                    , size: 17, align: 1, Ok(32)
                    , size: 32, align: 32, Ok(32)
                    , size: 256, align: 256, Ok(256)
                    };
        // Aligned allocations use alignment as block size.
        assert_size!(size: 16, align: 64, Ok(64));

        // Block orders.
        assert_order!{ size: 0, align: 1, Ok(0)
                     , size: 1, align: 1, Ok(0)
                     , size: 16, align: 16, Ok(0)
                     , size: 32, align: 32, Ok(1)
                     , size: 64, align: 64, Ok(2)
                     , size: 128, align: 128, Ok(3)
                     , size: 256, align: 256, Ok(4)
                    //  , size: 512, align: 512, Err
                     };
        assert!(heap.alloc_order(&Layout::from_size_align(512,512)).is_err());
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
        let heap = Heap::new( mem
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
        let mut heap = Heap::new( mem
                                              , &mut free_lists
                                              , HEAP_SIZE );

        let block_128_0 = heap.alloc(Layout::from_size_align(128, 128));
        assert_eq!(Ok(mem.offset(0)), block_128_0);

        heap.dealloc(block_128_0.unwrap(),Layout::from_size_align(128, 128));

        let block_16_0 = heap.alloc(Layout::from_size_align(8, 8));
        assert_eq!(Ok(mem), block_16_0);

        let bigger_than_heap = heap.alloc(Layout::from_size_align(4096, HEAP_SIZE));
        assert!(bigger_than_heap.is_err());

        let bigger_than_free = heap.alloc(Layout::from_size_align(HEAP_SIZE, HEAP_SIZE));
        assert!(bigger_than_free.is_err());

        let block_16_1 = heap.alloc(Layout::from_size_align(8, 8));
        assert_eq!(Ok(mem.offset(16)), block_16_1);

        let block_16_2 = heap.alloc(Layout::from_size_align(8, 8));
        assert_eq!(Ok(mem.offset(32)), block_16_2);

        let block_32_2 = heap.alloc(Layout::from_size_align(32, 32));
        assert_eq!(Ok(mem.offset(64)), block_32_2);

        let block_16_3 = heap.alloc(Layout::from_size_align(8, 8))      ;
        assert_eq!(Ok(mem.offset(48)), block_16_3);

        // let too_fragmented = heap.alloc(Layout::from_size_align(64, 64);
        // assert_eq!(None, too_fragmented);
        heap.dealloc(block_16_0.unwrap(), Layout::from_size_align(8, 8));
        heap.dealloc(block_16_3.unwrap(), Layout::from_size_align(8, 8));
        heap.dealloc(block_16_1.unwrap(), Layout::from_size_align(8, 8));
        heap.dealloc(block_16_2.unwrap(), Layout::from_size_align(8, 8));

        heap.dealloc(block_32_2.unwrap(), Layout::from_size_align(32, 32));

        let block_128_0 = heap.alloc(Layout::from_size_align(128, 128));
        assert_eq!(Ok(mem.offset(0)), block_128_0);

        let block_128_1 = heap.alloc(Layout::from_size_align(128, 128));
        assert_eq!(Ok(mem.offset(128)), block_128_1);

        heap.dealloc(block_128_1.unwrap(), Layout::from_size_align(128, 128));
        heap.dealloc(block_128_0.unwrap(), Layout::from_size_align(128, 128));

        // And allocate the whole heap, just to make sure everything
        // got cleaned up correctly.
        let block_256_0 = heap.alloc(Layout::from_size_align(256, 256)).unwrap();
        assert_eq!(mem.offset(0), block_256_0);

        free(mem);
    }
}
