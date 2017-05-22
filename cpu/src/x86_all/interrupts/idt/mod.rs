//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the `x86` and `x86_64` Interrupt Descriptor Table.
#![warn(missing_docs)]
use core::{default, mem, convert, ops};

use ::dtable::DTable;
use ::PrivilegeLevel;

/// Number of entries in the system's Interrupt Descriptor Table.
pub const ENTRIES: usize = 256;

#[cfg(test)] mod tests;

//==------------------------------------------------------------------------==
// IDT Gates
#[cfg(target_arch = "x86")]    #[path = "gate32.rs"] pub mod gate;
#[cfg(target_arch = "x86_64")] #[path = "gate64.rs"] pub mod gate;
pub use self::gate::*;

bitflags! {
    /// Bitflags field in an IDT gate.
    ///
    /// The structure of the flags field is as follows:
    ///
    /// ```ignore
    ///   7                           0
    /// +---+---+---+---+---+---+---+---+
    /// | P |  DPL  | S |    GateType   |
    /// +---+---+---+---+---+---+---+---+
    /// ```
    ///
    /// Fields have the following meanings:
    ///
    ///  + `P`: One bit, indicating if the ISR is present. Set to 0 for unused
    ///         interrupts.
    ///  + `DPL`: Two bits, indicating the escriptor's priveliege level as an
    ///           integer, with zero being Ring 0.
    ///  + `S`: One bit, set if the descriptor refers to an interrupt in the
    ///         storage segment.
    ///  + `GateType`: Four bits, indicating the type of the interrupt with the
    ///               following values (architecture-dependent):
    ///    - `0101`: 80386 32-bit task gate
    ///    - `0110`: 80286 16-bit interrupt gate
    ///    - `0111`: 80286 16-bit trap gate
    ///    - `1110`: 80386 32-bit interrupt gate
    ///    - `1111`: 80386 32-bit trap gate
    ///
    /// For more information, refer to the _Intel® 64 and IA-32 Architectures
    /// Software Developer’s Manual_, Vol. 3A, section 6.11, "IDT Descriptors";
    /// and to the OS Dev Wiki
    /// [article](http://wiki.osdev.org/Interrupts_Descriptor_Table)
    /// "Interrupts Descriptor Table".
    pub flags GateFlags: u8 {
        /// Set to 0 for unused interrupts.
        ///
        /// Indicates whether or not this gate is present.
        /// An interrupt on a non-present gate will trigger a
        /// General Protection Fault.
        const PRESENT       = 0b1000_0000

      , /// Bit indicating that the descriptor priveliege level is Ring 0
        const DPL_RING_0    = 0b0000_0000
      , /// Bit indicating that the descriptor priveliege level is Ring 1
        const DPL_RING_1    = 0b0010_0000
      , /// Bit indicating that the descriptor priveliege level is Ring 2
        const DPL_RING_2    = 0b0100_0000
      , /// Bit indicating that the descriptor priveliege level is Ring 3
        const DPL_RING_3    = 0b0110_0000
      , /// Descriptor priveliege level bitfield.
        const DPL           = DPL_RING_0.bits | DPL_RING_1.bits |
                              DPL_RING_2.bits | DPL_RING_3.bits

      , /// Storage segment flag.
        ///
        /// Set to 0 for interrupt gates.
        const SEGMENT       = 0b0001_0000
      , /// Set if this `Gate` points to a 32-bit ISR.
        const LONG_MODE     = 0b0000_1000

      , /// Set if this is an interrupt gate.
        const INT_GATE_16   = 0b0000_0110
      , /// Set if this is an interrupt gate and points to a 32-bit ISR.
        const INT_GATE_32   = INT_GATE_16.bits | LONG_MODE.bits
      , /// Set if this is a trap gate.
        const TRAP_GATE_16  = 0b0000_0111
      , /// Set if this is a trap gate that points to a 32-bit ISR
        const TRAP_GATE_32  = TRAP_GATE_16.bits | LONG_MODE.bits
      , /// Set if this is a 32-bit task gate.
        const TASK_GATE_32  = 0b0000_0101 | LONG_MODE.bits
    }
}

impl GateFlags {
    /// Returns true if this `Gate` is a trap gate
    #[inline] pub fn is_trap(&self) -> bool {
        self.contains(TRAP_GATE_16)
    }

    /// Returns true if this `Gate` points to a present ISR
    #[inline] pub fn is_present(&self) -> bool {
        self.contains(PRESENT)
    }

    /// Sets the present bit for this gate
    #[inline] pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present { self.insert(PRESENT) }
        else { self.remove(PRESENT) }
        self
    }

    /// Checks the gate's privilege
    #[inline] pub fn get_dpl(&self) -> PrivilegeLevel {
        unsafe { mem::transmute((*self & DPL).bits as u16 >> 5) }
    }

    /// Sets the privilege level of the gate
    pub fn set_dpl(&mut self, dpl: PrivilegeLevel) -> &mut Self {
        self.insert(GateFlags::from_bits_truncate((dpl as u8) << 5));
        self
    }

}


impl Default for GateFlags {
    fn default() -> Self { GateFlags { bits: 0 } }
}

//==------------------------------------------------------------------------==
use super::ErrorCodeHandler;
//  IDT implementation
/// An Interrupt Descriptor Table
///
/// The IDT is either 64-bit or 32-bit.
pub struct Idt {
    pub divide_by_zero: Gate
  , /// debug interrupt handler - reserved
    debug: Gate
  , pub nmi: Gate
  , pub breakpoint: Gate
  , pub overflow: Gate
  , pub bound_exceeded: Gate
  , pub undefined_opcode: Gate
  , pub device_not_available: Gate
  , pub double_fault: Gate<ErrorCodeHandler>
  , pub coprocessor_segment_overrun: Gate<ErrorCodeHandler>
  , pub invalid_tss: Gate<ErrorCodeHandler>
  , pub segment_not_present: Gate<ErrorCodeHandler>
  , pub stack_segment_fault: Gate<ErrorCodeHandler>
  , pub general_protection_fault: Gate<ErrorCodeHandler>
  , pub page_fault: Gate<ErrorCodeHandler>
  , _reserved: Gate
  , pub floating_point_error: Gate
  , pub alignment_check: Gate<ErrorCodeHandler>
  , pub machine_check: Gate
  , pub simd_fp_exception: Gate
  , /// user-defined interrupts
    pub interrupts: [Gate; ENTRIES - super::NUM_EXCEPTIONS]
}

impl Default for Idt {
    #[inline]
    fn default() -> Self {
        Idt {
            interrupts: [Default::default(); ENTRIES - super::NUM_EXCEPTIONS]
            , ..Default::default()
        }
    }
}

impl ops::Index<usize> for Idt {
    type Output = Gate;

    #[inline]
    fn index(&self, index: usize) -> &Gate {
        unsafe {
            &mem::transmute::<&Self, &[Gate; ENTRIES]>(self)[index]
        }
    }
}

impl ops::IndexMut<usize> for Idt {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut mem::transmute::<&mut Self, &mut [Gate; ENTRIES]>(self)[index]
        }
    }
}

impl Idt {

    /// Construct a new IDT with all interrupt gates set to [`absent`].
    ///
    /// [`absent`]: struct.Gate.absent.html
    pub const fn new() -> Self {
        Idt { divide_by_zero: Gate::absent()
            , debug: Gate::absent()
            , nmi: Gate::absent()
            , breakpoint: Gate::absent()
            , overflow: Gate::absent()
            , bound_exceeded: Gate::absent()
            , undefined_opcode: Gate::absent()
            , device_not_available: Gate::absent()
            , double_fault: Gate::absent()
            , coprocessor_segment_overrun: Gate::absent()
            , invalid_tss: Gate::absent()
            , segment_not_present: Gate::absent()
            , stack_segment_fault: Gate::absent()
            , general_protection_fault: Gate::absent()
            , page_fault: Gate::absent()
            , _reserved: Gate::absent()
            , floating_point_error: Gate::absent()
            , alignment_check: Gate::absent()
            , machine_check: Gate::absent()
            , simd_fp_exception: Gate::absent()
            , interrupts: [Gate::absent(); ENTRIES - super::NUM_EXCEPTIONS]
            }
    }

    /// Enable interrupts
    pub unsafe fn enable_interrupts() { asm!("sti") }
    /// Disable interrupts
    pub unsafe fn disable_interrupts() { asm!("cli") }

    /// Add a new interrupt gate pointing to the given handler
    #[inline]
    pub fn add_handler<Handler>( &mut self
                               , idx: usize
                               , handler: Handler)
                               -> &mut Self
    where Gate: convert::From<Handler> {
        self.add_gate(idx, Gate::from(handler))
    }

    /// Add a [`Gate`](struct.Gate.html) to the IDT.
    #[inline]
    pub fn add_gate(&mut self, idx: usize, gate: Gate) -> &mut Self {
        self[idx] = gate;
        self
    }

}

impl DTable for Idt {
    type Entry = Gate;

    #[inline] fn entry_count(&self) -> usize { ENTRIES }

    #[inline] fn load(&'static self) {
        unsafe {
            asm!(  "lidt ($0)"
                :: "r"(&self.get_ptr())
                :  "memory" );
        }
        kinfoln!(dots: " . . ", target: "Loading IDT", "[ OKAY ]");
    }
}
