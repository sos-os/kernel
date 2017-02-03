//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use memory::PAddr;
use super::{Address, Allocator, AllocErr, Layout};

/// A simple bump pointer allocator.
///
/// This allocator has few "moving parts" and is very fast. However, it doesn't
/// allow objects to be deallocated. We use this allocator for early kernel
/// objects before we can set up a more sophisticated heap allocator.
#[derive(Debug)]
pub struct BumpPtr { start: PAddr
                   , end: PAddr
                   , ptr: *mut u8
                   }

impl BumpPtr {
    pub const fn new(start: PAddr, end: PAddr) -> Self {
        BumpPtr { start: start
                , end: end
                , ptr: start.as_mut_ptr()
                }
    }
}

unsafe impl Allocator for BumpPtr {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<Address, AllocErr> {
        unimplemented!()
    }
    unsafe fn dealloc(&mut self, ptr: Address, layout: Layout) {
        // just leak it
    }
}
