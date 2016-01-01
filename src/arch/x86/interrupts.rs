//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 32-bit Interrupt Descriptor Table implementation.
//! This is a WORK IN PROGRESS

use core::mem;
use spin::Mutex;

#[path = "../x86_all/interrupts.rs"] mod interrupts_all;
pub use self::interrupts_all::*;

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
struct Gate32 { /// bits 0 - 15 of the offset
              offset_lower: u16
            , /// code segment selector (GDT or LDT)
              selector: u16
            , /// always zero
              zero: u8
            , /// indicates the gate's type and attributes.
              /// the second half indicates the type:
              ///   + `0b1100`: Call gate
              ///   + `0b1110`: Interrupt gate
              ///   + `0b1111`: Trap Gate
              type_attr: GateType
            , /// bits 16 - 31 of the offset
              offset_upper: u16
            }

impl Gate32 {
    /// Creates a new IDT gate marked as `absent`.
    ///
    /// This is basically just for filling the new IDT table
    /// with valid (but useless) gates upon init.
    const fn absent() -> Self {
        Gate32 { offset_lower: 0
               , selector: 0
               , zero: 0
               , type_attr: GateType::Absent
               , offset_upper: 0
               }
    }
}

impl Gate for Gate32 {

    /// Creates a new IDT gate pointing at the given handler function.
    fn new(handler: Handler) -> Self {
        unsafe { // trust me on this.
                 // `mem::transmute()` is glorious black magic
            let (low, mid, high): (u16, u16)
                = mem::transmute(handler);

            Gate32 { offset_lower: low
                   , selector: gdt32_offset
                   , zero: 0
                   , type_attr: GateType::Interrupt
                   , offset_mid: mid
                   , offset_upper: high
                   , reserved: 0
                   }
        }
    }
}


struct Idt32([Gate32; IDT_ENTRIES]);

impl Idt for Idt32 {
    type Ptr = Idt32Ptr;
    /// Get the IDT pointer struct to pass to `lidt`
    fn get_ptr(&self) -> Self::Ptr {
        Idt32Ptr { limit: (mem::size_of::<Gate32>() * IDT_ENTRIES) as u16
                 , base:  (&self.0[0] as *const Gate32) as u32
                 }
    }

    /// Enable interrupts
    unsafe fn enable_interrupts() {
        asm!("sti" :::: "volatile")
    }

    /// This is just a wrapper for prettiness reasons.
    #[inline]
    unsafe fn load(&self) {
        self.get_ptr()
            .load()
    }


}

/// This is the format that `lidt` expects for the pointer to the IDT.
/// ...apparently.
#[repr(C, packed)]
struct Idt32Ptr { limit: u16
                , base: u32
                }

impl IdtPtr for Idt32Ptr {
    /// Load the IDT at the given location.
    /// This just calls `lidt`.
    pub unsafe fn load(&self) {
        asm!(  "lidt ($0)"
            :: "{eax}"(self)
            :: "volatile" );
    }
}

/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt32>
    = Mutex::new(Idt32([Gate32::absent(); IDT_ENTRIES]));
