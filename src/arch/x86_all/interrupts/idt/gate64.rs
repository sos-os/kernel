//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 64-bit IDT gate implementation
use arch::cpu::segment;
use super::{Handler, GateType};

use core::{convert, mem};

extern {
    /// Offset of the 64-bit GDT main code segment.
    /// Exported by `boot.asm`
    static gdt64_offset: u16;
}

/// An IDT entry is called a gate.
///
/// Based on [code](http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure)
/// from the OS Dev Wiki.
///
/// Refer also to "6.14.1 64-Bit Mode IDT"  and "Table 3-2. System-Segment and
/// Gate-Descriptor Types" in the _Intel® 64 and IA-32 Architectures
/// Software Developer’s Manual_
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
                   pub offset_mid: u16
                 , /// bits 32 - 63 of the offset
                   pub offset_upper: u32
                 , /// always zero (according to the spec, this is "reserved")
                   _reserved: u32
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
            , selector: segment::Selector::from_raw(0)
            , _zero: 0
            , type_attr: GateType::Absent
            , offset_mid: 0
            , offset_upper: 0
            , _reserved: 0
            }
    }

}

impl convert::From<Handler> for Gate {
    fn from(handler: Handler) -> Self {
        let (low, mid, high): (u16, u16, u32)
            // trust me on this, `mem::transmute()` is glorious black magic
            = unsafe { mem::transmute(handler) };

        Gate { offset_lower: low
             , selector: segment::Selector::from_raw(gdt64_offset)
             , _zero: 0
             // Bit 7 is the present bit
             // Bits 4-0 indicate this is an interrupt gate
             , type_attr: GateType::Interrupt
             , offset_mid: mid
             , offset_upper: high
             , _reserved: 0
             }
    }
}
