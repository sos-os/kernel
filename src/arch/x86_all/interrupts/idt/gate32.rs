//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 32-bit IDT gate implementation
use arch::cpu::segment;
use super::{Handler, GateType};

use core::mem::transmute;

extern {
    /// Offset of the 32-bit GDT main code segment.
    /// Exported by `boot.asm`
    static gdt32_offset: u16;
}

/// An IDT entry is called a gate.
///
/// Based on code from the OS Dev Wiki
/// http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure
#[repr(C, packed)]
#[derive(Copy,Clone)]
pub struct Gate { /// bits 0 - 15 of the offset
                 pub offset_lower: u16
               , /// code segment selector (GDT or LDT)
                 pub selector: segment::Selector
               , /// always zero
                 _zero: u8
               , /// indicates the gate's type and attributes.
                 /// the second half indicates the type:
                 ///   + `0b1100`: Call gate
                 ///   + `0b1110`: Interrupt gate
                 ///   + `0b1111`: Trap Gate
                 pub type_attr: GateType
               , /// bits 16 - 31 of the offset
                 pub offset_upper: u16
               }

impl Gate {

   /// Creates a new IDT gate marked as `absent`.
   ///
   /// This is basically just for filling the new IDT table
   /// with valid (but useless) gates upon init.
   ///
   /// This would be in the `Gate` trait, but this has to be a `const fn` so
   /// that it can be usedm in static initializers, and trait functions cannot
   /// be `const`.
   ///
   /// Actually triggering an absent interrupt will send a General Protection
   /// fault (13).
    pub const fn absent() -> Self {
       Gate { offset_lower: 0
            , selector: 0
            , _zero: 0
            , type_attr: GateType::Absent
            , offset_upper: 0
            }
    }

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    pub fn from_handler(handler: Handler) -> Self {
        // trust me on this, `mem::transmute()` is glorious black magic
        let (low, high): (u16, u16) = unsafe { transmute(handler) };

        Gate { offset_lower: low
             , selector: gdt32_offset
             , _zero: 0
             , type_attr: GateType::Interrupt
             , offset_upper: high
             , _reserved: 0
             }
    }

    ///  Creates a new IDT gate from a raw reference to a handler.
    ///
    ///  This should probably not be used ever.
    pub unsafe fn from_raw(handler: *const u8) -> Self {
        let (low, high): (u16, u16) = transmute(handler as u64);

        Gate { offset_lower: low
             , selector: segment::Selector::from_raw(gdt32_offset)
             , _zero: 0
             , type_attr: GateType::Interrupt
             , offset_upper: high
             , _reserved: 0
             }
    }
}
