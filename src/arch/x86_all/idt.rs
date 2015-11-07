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

pub trait Idt {
    type Ptr: IdtPtr;

    fn get_ptr(&self) -> Self::Ptr;

    unsafe fn load(&self);
    unsafe fn enable_interrupts();
}

pub trait IdtPtr {
    unsafe fn load(&self);
}
