//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Kernel parameters
//!
#[deny(missing_docs)]

// use memory::paging::PageRange;
use memory::PAddr;

/// Parameters used during the init process
// TODO: can this be factored out & used in crates?
//          - eliza, 1/21/2017
pub struct InitParams {
    /// The base of the kernel memory range
    // TODO: rewrite to use FrameRange once that's on master
    pub kernel_base: PAddr
  , /// The top of the kernel memory range
    // TODO: rewrite to use FrameRange once that's on master
    pub kernel_top: PAddr
  , /// The base of the memory range for the kernel heap
    // TODO: rewrite to use FrameRange once that's on master
    pub heap_base: PAddr
  , /// The top of the memory range to use for the kernel heap
    // TODO: rewrite to use FrameRange once that's on master
    pub heap_top: PAddr
}
