//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS init parameters
//!
//! This crate is intended to facilitate the sharing of initialization
//! parameters between "higher-level" SOS subcrates (such as [`alloc`] and
//! [`paging`]) in a platform-independent way.
//!
//! [`alloc`](../alloc)
//! [`paging`](../paging)
#![no_std]
#![deny(missing_docs)]

extern crate memory;
extern crate elf;
// use memory::paging::PageRange;
use memory::PAddr;

/// If we are on x86_64 or armv7 this uses the 64-bit ELF word
#[cfg(target_pointer_width = "64")]
pub type ElfSections<'a> = elf::section::Sections<'a, u64>;

/// If we are on x86, this uses the 32-bit ELF word
#[cfg(target_pointer_width = "32")]
pub type ElfSections<'a> = elf::section::Sections<'a, u32>;

/// Parameters used during the init process
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

impl InitParams {
    /// Returns an iterator over the kernel's ELF sections
    // TODO: is this cross-platform? are we using ELF on all our supported
    //       architectures? i think we are, but we should ensure this is the
    //       case...
    //          – eliza, 1/22/2017
    pub fn elf_sections(&self) ->  ElfSections {
        unimplemented!()
    }

    /// Returns the start address of the multiboot info struct
    pub fn multiboot_start(&self) -> PAddr {
        unimplemented!()
    }

    /// Returns the end address of the multiboot info struct
    pub fn multiboot_end(&self) -> PAddr {
        unimplemented!()
    }
}
