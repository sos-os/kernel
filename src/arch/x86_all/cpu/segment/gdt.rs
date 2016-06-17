//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! The Global Descriptor Table (GDT) is used for configuring segmentation.
//!
//! As we use paging rather than segmentation for memory management, we do
//! not actually use the GDT, but some x86 functionality still require it
//! to be properly configured.
use super::Descriptor;
use arch::cpu::dtable::DTable;

const GDT_SIZE: usize = 512;

pub type Gdt = [Descriptor; GDT_SIZE];

impl DTable for Gdt {
    type Entry = Descriptor;

    /// Returns the number of Entries in the `DTable`.
    ///
    /// This is used for calculating the limit.
    #[inline(always)] fn entry_count(&self) -> usize { GDT_SIZE }

    /// Load the GDT table with the `lgdt` instruction.
    #[inline] unsafe fn load(&self) {
        asm!(  "lgdt ($0)"
            :: "r"(&self.get_ptr())
            :  "memory" );
    }
}
