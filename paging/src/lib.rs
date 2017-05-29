//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS Paging
//!
//! This is in its own crate so that it can depend on both the `memory` and
//! `alloc` crates.
#![feature(asm)]
#![feature(unique)]
#![feature(associated_consts, const_fn)]
#![feature(core_intrinsics)]
#![no_std]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;
#[macro_use] extern crate vga;
extern crate spin;

extern crate util;
extern crate memory;
extern crate alloc;
extern crate cpu;
extern crate elf;
extern crate params;

pub mod arch;
pub mod stack;
pub use self::arch::{kernel_remap, test_paging};

use memory::{Page, PAddr, PhysicalPage, VAddr, VirtualPage};
use alloc::{FrameAllocator, AllocErr};
use core::fmt;

pub type MapResult<T = ()> = Result<T, MapErr>;

#[derive(Clone)]
pub enum MapErr<P: Page + fmt::Debug = VirtualPage> {
    Alloc { message: &'static str, page: P, cause: AllocErr }
  , Other { message: &'static str, page: P, cause: &'static str }
  , TableNotFound { message: &'static str, page: VirtualPage, what: &'static str }
  , AlreadyInUse { message: &'static str, page: VirtualPage, frame: PhysicalPage }
}

impl<P> fmt::Debug for MapErr<P> where P: Page + fmt::Debug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

pub trait Mapper {
    type Flags;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    /// + `Some(PAddr)` containing the physical address corresponding to
    ///                 `vaddr`, if it is mapped.
    /// + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: VirtualPage) -> Option<PhysicalPage>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>( &mut self, page: VirtualPage, frame: PhysicalPage
             , flags: Self::Flags, alloc: &mut A )
             -> MapResult<()>
    where A: FrameAllocator;

    /// Identity map a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>( &mut self, frame: PhysicalPage
                      , flags: Self::Flags, alloc: &mut A )
                      -> MapResult<()>
    where A: FrameAllocator;

    /// Map the given `VirtualPage` to any free frame.
    ///
    /// This is like the fire and forget version of `map_to`: we just pick the
    /// first available free frame and map the page to it.
    ///
    /// # Arguments
    /// + `page`: the`VirtualPage` to map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to_any<A>( &mut self, page: VirtualPage
                    , flags: Self::Flags
                    , alloc: &mut A)
                    -> MapResult<()>
    where A: FrameAllocator;

    /// Unmap the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `FrameAllocator`.
    fn unmap<A>(&mut self, page: VirtualPage, alloc: &mut A) -> MapResult<()>
    where A: FrameAllocator;

}
