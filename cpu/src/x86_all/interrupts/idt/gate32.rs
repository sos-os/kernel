//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 32-bit IDT gate implementation
use ::segment;
use super::{Handler, GateFlags};

use core::mem::transmute;

extern {
    /// Offset of the 32-bit GDT main code segment.
    /// Exported by `boot.asm`
    #[link_name="gdt32_offset"]
    static GDT_OFFSET: u16;
}

/// An IDT entry is called a gate.
///
/// Based on [code](http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure)
/// from the OS Dev Wiki.
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
                 /// + `0b1100`: Call gate
                 /// + `0b1110`: Interrupt gate
                 /// + `0b1111`: Trap Gate
                 pub flags: GateFlags
               , /// bits 16 - 31 of the offset
                 pub offset_upper: u16
               }

impl GateFlags {

   /// Returns a new trap gate
   pub const fn new_trap() -> Self {
       GateFlags { bits: super::TRAP_GATE_16.bits | super::PRESENT.bits }
   }

   /// Returns a new interrupt gate
   pub const fn new_interrupt() -> Self {
       GateFlags { bits: super::INT_GATE_16.bits | super::PRESENT.bits }
   }

}

impl Gate {

   /// Creates a new IDT gate marked as `absent`.
   ///
   /// This is basically just for filling the new IDT table
   /// with valid (but useless) gates upon init.
   ///
   /// Actually triggering an absent interrupt will send a General Protection
   /// fault (13).
    pub const fn absent() -> Self {
       Gate { offset_lower: 0
            , selector: 0
            , _zero: 0
            , flags: GateFlags { bits: 0 }
            , offset_upper: 0
            }
    }

}

impl convert::From<Handler> for Gate {

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    fn from(handler: Handler) -> Self {
        unsafe {
            let (low, mid): (u16, u16) = mem::transmute(handler);

            Gate { offset_lower: low
                 , selector: segment::Selector::from_raw(GDT_OFFSET)
                 , _zero: 0
                 , type_attr: GateFlags::new_interrupt()
                 , offset_upper: high
                 , _reserved: 0
                 }
        }
    }
}
