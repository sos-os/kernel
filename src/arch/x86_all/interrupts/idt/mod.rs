//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the `x86` and `x86_64` Interrupt Descriptor Table.

use core::{fmt, ptr};

use arch::cpu::dtable::DTable;

extern {
    /// Array of interrupt handlers exported by ASM
    //  TODO: hopefully, we will not need this much longer.
    #[link_name = "isrs"]
    static ISRs: [*const u8; ENTRIES];
}

/// An interrupt handler function.
pub type Handler = unsafe extern "C" fn() -> ();

/// Number of entries in the system's Interrupt Descriptor Table.
pub const ENTRIES: usize = 256;

//==------------------------------------------------------------------------==
// IDT Gates
#[cfg(target_arch = "x86")]    #[path = "gate32.rs"] pub mod gate;
#[cfg(target_arch = "x86_64")] #[path = "gate64.rs"] pub mod gate;
pub use self::gate::*;

/// `x86` interrupt gate types.
///
/// Bit-and this with the attribute half-byte to produce the
/// `type_attr` field for a `Gate`
#[repr(u8)]
#[derive(Copy,Clone,Debug)]
pub enum GateType { Absent    = 0b0000_0000
                  , Interrupt = 0b1000_1110
                  , Call      = 0b1000_1100
                  , Trap      = 0b1000_1111
                  }

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self { GateType::Absent    => write!(f, "Absent")
                    , GateType::Interrupt => write!(f, "Interrupt")
                    , GateType::Call      => write!(f, "Call")
                    , GateType::Trap      => write!(f, "Trap")
                    }
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
    pub fn add_gate(&mut self, idx: usize, handler: Handler) -> &mut Self {
        self.0[idx] = Gate::from(handler);
        self
    }

    /// Add interrupt handlers exported by assembly to the IDT.
    pub unsafe fn add_handlers(&mut self) -> &mut Self {
        for (i, &handler_ptr) in ISRs.iter()
            .enumerate()
            .filter(|&(_, &ptr)| ptr != ptr::null() ) {
                self.0[i] = Gate::from(handler_ptr)
        }

        println!("{:<38}{:>40}", " . . Adding interrupt handlers to IDT"
             , "[ OKAY ]");
        self
    }

}

impl DTable for Idt {
    type Entry = Gate;

    #[inline] fn entry_count(&self) -> usize { ENTRIES }

    #[inline] unsafe fn load(&self) {
        asm!(  "lidt ($0)"
            :: "r"(&self.get_ptr())
            :  "memory" );
        println!("{:<38}{:>40}", " . . Loading IDT", "[ OKAY ]");
    }
}
