//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific memory management.
use core::mem;

use ::Addr;
// use super::paging::Page;
//pub mod table;
//pub mod entry;
// pub mod paging;

use core::ops;

pub const PAGE_SHIFT: u8 = 12;
/// The size of a page (4KiB), in bytes
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT; // 4k
/// The size of a large page (2MiB) in bytes
pub const LARGE_PAGE_SIZE: u64 = 1024 * 1024 * 2;
/// The size of a huge page (2GiB) in bytes
pub const HUGE_PAGE_SIZE: u64 = 1024 * 1024 * 1024;

extern {
    // TODO: It would be really nice if there was a less ugly way of doing
    // this... (read: after the Revolution when we add memory regions to the
    // heap programmatically.)
    #[link_name = "heap_base_addr"]
    pub static HEAP_BASE: PAddr;
    #[link_name = "heap_top_addr"]
    pub static HEAP_TOP: PAddr;
    // Of course, we will still need to export the kernel stack addresses like
    // this, but it would be nice if they could be, i dont know, not mut u8s
    // pointers, like God intended.
    #[link_name = "stack_base_addr"]
    pub static STACK_BASE: PAddr;
    #[link_name = "stack_top_addr"]
    pub static STACK_TOP: PAddr;
}

macro_attr! {
    /// A physical (linear) memory address is a 64-bit unsigned integer
    #[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Addr!(u64))]
    #[repr(C)]
    pub struct PAddr(u64);
}
//
// macro_attr! {
//     /// A frame (physical page)
//     #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Page!(PAddr) )]
//     pub struct PhysicalPage { pub number: u64 }
// }
