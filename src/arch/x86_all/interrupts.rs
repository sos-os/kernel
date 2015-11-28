//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the x86 and x86_64 Interrupt Descriptor Table.

use super::super::{DTablePtr, DTable};
use core::mem::size_of;
use core::fmt;

pub type Handler = unsafe extern "C" fn() -> ();
pub const IDT_ENTRIES: usize = 256;

/// x86 interrupt gate types.
///
/// Bit-and this with the attribute half-byte to produce the
/// `type_attr` field for a `Gate`
#[repr(u8)]
#[derive(Debug)]
pub enum GateType { Absent    = 0b0000_0000
                  , Interrupt = 0b1000_1110
                  , Call      = 0b1000_1100
                  , Trap      = 0b1000_1111
                  }

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self { &GateType::Absent    => write!(f, "Absent")
                   , &GateType::Interrupt => write!(f, "Interrupt")
                   , &GateType::Call      => write!(f, "Call")
                   , &GateType::Trap      => write!(f, "Trap")
                   }
    }
}

#[derive(Debug)]
pub struct ExceptionDescr { pub description: &'static str
                          , pub mnemonic: &'static str
                        //   , pub
}


/// x86 exceptions.
///
/// Taken from the list at
/// [http://wiki.osdev.org/Exceptions](http://wiki.osdev.org/Exceptions)
pub static EXCEPTIONS: &'static [&'static str] = &[
    "Divide-by-zero Error",
    "Debug",
    "Non-maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
];

pub trait Gate {
    fn from_handler(handler: Handler) -> Self;
}

// /// This is the format that `lidt` expects for the pointer to the IDT.
// /// ...apparently.
// #[repr(C, packed)]
// pub struct IdtPtr<I>
// where I: Idt { pub limit: u16
//              , pub base: *const I
//              }
//
// pub trait IdtPtrOps {
//     unsafe fn load(&self);
// }

pub trait InterruptContext {
    type Registers;

    fn err_no(&self) -> u32;
    fn int_id(&self) -> u32;
    fn registers(&self) -> Self::Registers;

    #[inline]
    unsafe fn exception(&self) -> &str {
        EXCEPTIONS[self.int_id() as usize]
    }


}

pub trait Idt: Sized {
    type Ctx: InterruptContext;
    type GateSize: Gate;

    /// Get the IDT pointer struct to pass to `lidt`
    fn get_ptr(&self) -> DTablePtr<Self> {
        DTablePtr { limit: (size_of::<Self::GateSize>() * IDT_ENTRIES) as u16
                  , base: self as *const Self
                  }
    }
    //
    // /// This is just a wrapper for prettiness reasons.
    // #[inline]
    // unsafe fn load(&self) {
    //     self.get_ptr()
    //         .load()
    // }

    /// Enable interrupts
    unsafe fn enable_interrupts() {
        asm!("sti" :::: "volatile")
    }

    fn disable_interrupts() {
        unsafe { asm!("cli" :::: "volatile"); }
    }

    fn add_gate(&mut self, idx: usize, handler: Handler);

    fn handle_cpu_exception(state: &Self::Ctx)  {
        // TODO: we can handle various types of CPU exception differently
        // TODO: make some nice debugging dumps
        unsafe {
            panic!( "CPU EXCEPTION {:#04x}: {}"
                  , state.err_no()
                  , state.exception() );
              }
    }

    extern "C" fn handle_interrupt(state: &Self::Ctx);
}
