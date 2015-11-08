//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the x86 and x86_64 Interrupt Descriptor Table.

pub type Handler = unsafe extern "C" fn() -> ();
pub const IDT_ENTRIES: usize = 256;

/// x86 interrupt gate types.
///
/// Bit-and this with the attribute half-byte to produce the
/// `type_attr` field for a `Gate`
#[repr(u8)]
pub enum GateType { Interrupt = 0b0000_1110
                  , Call      = 0b0000_1100
                  , Trap      = 0b0000_1111
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
    fn new(handler: Handler) -> Self;
}

/// This is the format that `lidt` expects for the pointer to the IDT.
/// ...apparently.
#[repr(C, packed)]
pub struct IdtPtr<I>
where I: Idt { pub limit: u16
             , pub base: *const I
             }

pub trait IdtPtrOps {
    unsafe fn load(&self);
}

pub trait Idt: Sized {
    type Ptr: IdtPtrOps;

    fn get_ptr(&self) -> Self::Ptr;

    /// This is just a wrapper for prettiness reasons.
    #[inline]
    unsafe fn load(&self) {
        self.get_ptr()
            .load()
    }

    /// Enable interrupts
    unsafe fn enable_interrupts() {
        asm!("sti" :::: "volatile")
    }

    fn disable_interrupts() {
        unsafe { asm!("cli" :::: "volatile"); }
    }

    fn add_gate(&mut self, idx: usize, handler: Handler);
}
