//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86 descriptor tables (IDT, GDT, or LDT)

use memory::PAddr;

/// A pointer to a descriptor table.
/// This is a format suitable
#[repr(C, packed)]
pub struct Pointer { /// the length of the descriptor table
                     pub limit: u16
                   , /// pointer to the region in memory
                     /// containing the descriptor table.
                     pub base: PAddr
                   }

/// A descriptor table (IDT or GDT)
pub trait DTable {
    unsafe fn load(&self);
}
