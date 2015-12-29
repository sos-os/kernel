//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
pub mod addr;
pub use self::addr::*;
use alloc::buddy;

extern {
    static mut HEAP_BASE: u8;
    static mut HEAP_TOP: u8;
}

#[inline] pub fn heap_base_addr() -> usize {
    unsafe { (&mut HEAP_BASE as *mut _) as usize }
}

#[inline] pub fn heap_top_addr() -> usize {
    unsafe { (&mut HEAP_TOP as *mut _) as usize }
}

static mut KERNEL_FREE_LISTS: [buddy::FreeList<'static>; 19]
    // TODO: I really wish there was a less awful way to do this...
    = [ buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      , buddy::FreeList::new()
      ];

pub unsafe fn init_heap() {
    let heap_base_ptr
        = &mut HEAP_BASE as *mut _;
    let heap_size
        = (&mut HEAP_TOP as *mut _) as usize - heap_base_ptr as usize;
    buddy::system::init_heap( heap_base_ptr
                            , &mut KERNEL_FREE_LISTS
                            , heap_size);
}
