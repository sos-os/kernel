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
}
