//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the `x86` and `x86_64` Interrupt Descriptor Table.

use core::mem;

use ::dtable::DTable;
use ::PrivilegeLevel;

/// An interrupt handler function.
pub type Handler = unsafe extern "C" fn() -> !;

/// Number of entries in the system's Interrupt Descriptor Table.
pub const ENTRIES: usize = 256;

#[cfg(test)] mod tests;

//==------------------------------------------------------------------------==
// IDT Gates
#[cfg(target_arch = "x86")]    #[path = "gate32.rs"] pub mod gate;
#[cfg(target_arch = "x86_64")] #[path = "gate64.rs"] pub mod gate;
pub use self::gate::*;

bitflags! {
    pub flags GateFlags: u8 {
        /// Indicates whether or not this gate is present.
        /// An interrupt on a non-present gate will trigger a
        /// General Protection Fault.
        const PRESENT       = 0b1000_0000

      , const DPL_RING_0    = 0b0000_0000
      , const DPL_RING_1    = 0b0010_0000
      , const DPL_RING_2    = 0b0100_0000
      , const DPL_RING_3    = 0b0110_0000
      , const DPL           = DPL_RING_0.bits | DPL_RING_1.bits |
                              DPL_RING_2.bits | DPL_RING_3.bits

      , const SEGMENT       = 0b0001_0000
      , const LONG_MODE     = 0b0000_1000

      , const INT_GATE_16   = 0b0000_0110
      , const INT_GATE_32   = INT_GATE_16.bits | LONG_MODE.bits
      , const TRAP_GATE_16  = 0b0000_0111
      , const TRAP_GATE_32  = TRAP_GATE_16.bits | LONG_MODE.bits
      , const TASK_GATE_32  = 0b0000_0101 | LONG_MODE.bits
    }
}

impl GateFlags {
    #[inline] pub fn is_trap(&self) -> bool {
        self.contains(TRAP_GATE_16)
    }

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

//==------------------------------------------------------------------------==
//  IDT implementation
/// An Interrupt Descriptor Table
///
/// The IDT is either 64-bit or 32-bit.
pub struct Idt([Gate; ENTRIES]);

impl Idt {

    /// Construct a new IDT with all interrupt gates set to `absent`.
    pub const fn new() -> Self {
        Idt([Gate::absent(); ENTRIES])
    }

    /// Enable interrupts
    pub unsafe fn enable_interrupts() { asm!("sti") }
    /// Disable interrupts
    pub unsafe fn disable_interrupts() { asm!("cli") }

    /// Add a new interrupt gate pointing to the given handler
    #[inline]
    pub fn add_handler(&mut self, idx: usize, handler: Handler) -> &mut Self {
        self.add_gate(idx, Gate::from(handler))
    }

    #[inline]
    pub fn add_gate(&mut self, idx: usize, gate: Gate) -> &mut Self {
        self.0[idx] = gate;
        self
    }

    ///// Add interrupt handlers exported by assembly to the IDT.
    //pub unsafe fn add_handlers(&mut self) -> &mut Self {
    //    for (i, &handler_ptr) in ISRs.iter()
    //        .enumerate()
    //        .filter(|&(_, &ptr)| ptr != ptr::null() ) {
    //            self.0[i] = Gate::from(handler_ptr)
    //    }
    //
    //    println!("{:<38}{:>40}", " . . Adding interrupt handlers to IDT"
    //         , "[ OKAY ]");
    //    self
    //}

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
        infoln!(dots: " . . ", "Loading IDT", status: "[ OKAY ]");
    }
}
