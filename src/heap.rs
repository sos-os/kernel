//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use alloc::buddy;
use params::InitParams;

/// Initialise the kernel heap.
//  TODO: this is the Worst Thing In The Universe. De-stupid-ify it.
pub unsafe fn initialize<'a>(params: &InitParams) -> Result<&'a str, &'a str> {
    let heap_base_ptr = params.heap_base.as_mut_ptr();
    let heap_size: u64 = (params.heap_top - params.heap_base).into();
    buddy::system::init_heap(heap_base_ptr, heap_size as usize);
    Ok("[ OKAY ]")
}
