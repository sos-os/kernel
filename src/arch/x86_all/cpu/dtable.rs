//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! `x86` and `x86_64` descriptor tables (IDT, GDT, or LDT)

use memory::PAddr;
use core::mem::size_of;

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
pub trait DTable: Sized {
    /// Get the IDT pointer struct to pass to `lidt` or `lgdt`
    fn get_ptr(&self) -> Pointer {
        Pointer {
            limit: size_of::<Self>() as u16
          , base: PAddr::from(self as *const _)
        }
    }

    /// Load the descriptor table with the appropriate load instruction
    unsafe fn load(&self);
}
