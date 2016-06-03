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
    type Entry: Sized;

    /// Get the IDT pointer struct to pass to `lidt` or `lgdt`
    ///
    /// This expects that the object implementing `DTable` not contain
    /// additional data before or after the actual `DTable`, if you wish
    /// to attach information to a descriptor table besides the array of
    /// entries that it consists of, it will be necessary to encose the
    /// descriptor table in another `struct` or `enum` type.
    //  TODO: can we have an associated `Entry` type + a function to get the
    //        number of entries in the DTable, instead? that way, we could
    //        calculate the limit using that information, allowing Rust code
    //        to place more variables after the array in the DTable structure.
    //
    //        If we wanted to be really clever, we could probably also have a
    //        method to get a pointer to a first entry (or enforce that the
    //        DTable supports indexing?) and then we could get a pointer only
    //        to the array segment of the DTable, while still allowing variables
    //        to be placed before/after the array.
    //
    //        I'm not sure if we actually want to support this – is there really
    //        a use-case for it? I suppose it would also make our size calc.
    //        more correct in case Rust ever puts additional data around a
    //        DTable rray, but I imagine it will probably never do that...
    //              – eliza, 06/03/2016
    //
    fn get_ptr(&self) -> Pointer {
        Pointer {
            limit: (size_of::<Self::Entry>() * self.entry_count()) as u16
          , base: PAddr::from(self as *const _)
        }
    }

    /// Returns the number of Entries in the `DTable`.
    ///
    /// This is used for calculating the limit.
    fn entry_count(&self) -> usize;

    /// Load the descriptor table with the appropriate load instruction
    unsafe fn load(&self);
}
