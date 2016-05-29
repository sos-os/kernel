//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 64-bit Interrupt Descriptor Table implementation.
//!
//! Refer to section 6.10 of the _Intel® 64 and IA-32 Architectures
//! Software Developer’s Manual_ for more information.
use core::mem;
use core::ptr;

use spin::Mutex;

use super::{Registers, dtable, segment};
use super::dtable::DTable;
use io::keyboard;

#[path = "../../x86_all/interrupts.rs"] mod interrupts_all;
#[path = "../../x86_all/pics.rs"] pub mod pics;
pub use self::interrupts_all::*;

//==------------------------------------------------------------------------==
// Interface into ASM interrupt handling
extern {
    /// Offset of the 64-bit GDT main code segment.
    /// Exported by `boot.asm`
    static gdt64_offset: u16;

    /// Array of interrupt handlers from ASM
    static interrupt_handlers: [*const u8; IDT_ENTRIES];
}

/// State stored when handling an interrupt.
#[allow(dead_code)]
#[repr(C, packed)]
pub struct InterruptCtx64 { /// callee-saved registers
                            pub registers: Registers
                          , /// interrupt ID number
                            pub int_id:  u32
                          , __pad_1: u32
                          , /// error number
                            pub err_no:  u32
                          , __pad_2: u32
                          }

impl InterruptContext for InterruptCtx64 {
    type Registers = Registers;
    // All these inline functions are basically just faking
    // object orientation in a way the Rust compiler understands
    #[inline] fn registers(&self) -> Self::Registers { self.registers }
    #[inline] fn err_no(&self) -> u32 { self.err_no }
    #[inline] fn int_id(&self) -> u32 { self.int_id }
}


//==------------------------------------------------------------------------==
// 64-bit implementation of the IDT gate trait

/// An IDT entry is called a gate.
///
/// Based on code from the OS Dev Wiki
/// http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure
///
/// Refer also to "6.14.1 64-Bit Mode IDT"  and "Table 3-2. System-Segment and
/// Gate-Descriptor Types" in the _Intel® 64 and IA-32 Architectures
/// Software Developer’s Manual_
#[repr(C, packed)]
#[derive(Copy,Clone)]
pub struct Gate64 { /// bits 0 - 15 of the offset
                    pub offset_lower: u16
                  , /// code segment selector (GDT or LDT)
                    pub selector: segment::Selector
                  , /// always zero
                    zero: u8
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
                    reserved: u32
                  }

impl Gate64 {
    /// Creates a new IDT gate marked as `absent`.
    ///
    /// This is basically just for filling the new IDT table
    /// with valid (but useless) gates upon init.
    ///
    /// This would be in the `Gate` trait, but this has to be a `const fn` so
    /// that it can be usedm in static initializers, and trait functions cannot
    /// be `const`.
    ///
    /// Actually triggering an absent interrupt will send a General Protection
    /// fault (13).
    const fn absent() -> Self {
        Gate64 { offset_lower: 0
               , selector: segment::Selector::from_raw(0)
               , zero: 0
               , type_attr: GateType::Absent
               , offset_mid: 0
               , offset_upper: 0
               , reserved: 0
               }
    }
}

impl Gate for Gate64 {

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    fn from_handler(handler: Handler) -> Self {
        unsafe { // trust me on this.
                 // `mem::transmute()` is glorious black magic
            let (low, mid, high): (u16, u16, u32)
                = mem::transmute(handler);

            Gate64 { offset_lower: low
                   , selector: segment::Selector::from_raw(gdt64_offset)
                   , zero: 0
                   // Bit 7 is the present bit
                   // Bits 4-0 indicate this is an interrupt gate
                   , type_attr: GateType::Interrupt
                   , offset_mid: mid
                   , offset_upper: high
                   , reserved: 0
                   }
        }
    }

    ///  Creates a new IDT gate from a raw reference to a handler.
    ///
    ///  This should probably not be used ever.
    unsafe fn from_raw(handler: *const u8) -> Self {
        let (low, mid, high): (u16, u16, u32)
            = mem::transmute(handler as u64);

        Gate64 { offset_lower: low
               , selector: segment::Selector::from_raw(gdt64_offset)
               , zero: 0
               , type_attr: GateType::Interrupt
               , offset_mid: mid
               , offset_upper: high
               , reserved: 0
               }
    }
}

//==-------------------------------------------------------------------------==
// 64-bit implementation of the IDT trait
struct Idt64([Gate64; IDT_ENTRIES]);

impl Idt for Idt64 {
    type Ctx = InterruptCtx64;
    type GateSize = Gate64;
    //type PtrSize = u64;

    /// Add an entry for the given handler at the given index
    fn add_gate(&mut self, index: usize, handler: Handler) {
        self.0[index] = Gate64::from_handler(handler)
    }

}

impl Idt64 {
    /// Add interrupt handlers exported by assembly to the IDT.
    fn add_handlers(&mut self) -> &mut Self {
        for (i, &h_ptr) in interrupt_handlers.iter()
            .enumerate()
            .filter(|&(_, &h_ptr)| h_ptr != ptr::null() ) {
                unsafe { self.0[i] = Gate64::from_raw(h_ptr) }
        }

        println!("{:<38}{:>40}", " . . Adding interrupt handlers to IDT"
             , "[ OKAY ]");
        self
    }
}

//==--------------------------------------------------------------------------==
// Top-level interrupt handling
/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt64>
    = Mutex::new(Idt64([Gate64::absent(); IDT_ENTRIES]));

/// Kernel interrupt-handling function.
///
/// Assembly interrupt handlers call into this, and it dispatches interrupts to
/// the appropriate consumers.
#[no_mangle]
pub unsafe extern "C" fn handle_interrupt(state: &InterruptCtx64) {
    let id = state.int_id();
    match id {
        // interrupts 0 - 16 are CPU exceptions
        0x00...0x0f => Idt64::handle_cpu_exception(state)
        // System timer
      , 0x20 => { /* TODO: make this work */ }
        // Keyboard
      , 0x21 => {
          // TODO: dispatch keypress event to subscribers (NYI)
            if let Some(input) = keyboard::read_char() {
                if input == '\r' {
                    println!("");
                } else {
                    print!("{}", input);
                }
            }
        }
        // Loonix syscall vector
      , 0x80 => { // TODO: currently, we do nothing here, do we want
                  // our syscalls on this vector as well?
        }
      , _ => panic!("Unknown interrupt: #{} Sorry!", id)
    }
    // send the PICs the end interrupt signal
    pics::end_pic_interrupt(id as u8);
}


/// Initialize interrupt handling.
///
/// This function initializes the PICs, populates the IDT with interrupt
/// handlers, loads the IDT pointer, and enables interrupts.
///
/// This is called from the kernel during the init process.
pub unsafe fn initialize() {
    // println!(" . Enabling interrupts:" );
    // println!( " . . Initialising PICs {:>40}"
    //         , pics::initialize().unwrap_or("[ FAIL ]") );
    pics::initialize();
    IDT.lock()
       .add_handlers()
       .load();                 // Load the IDT pointer
    // print!("Testing interrupt handling...");
    // asm!("int $0" :: "N" (0x80));
    // println!("   [DONE]");
    Idt64::enable_interrupts(); // enable interrupts

}
