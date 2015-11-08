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
use spin::Mutex;

#[path = "../x86_all/interrupts.rs"] mod interrupts_all;
pub use self::interrupts_all::*;

extern {
    /// Offset of the 64-bit GDT main code segment.
    /// Exported by `boot.asm`
    static gdt64_offset: u16;

    /// Array of interrupt handlers from ASM
    static int_handlers: [Option<unsafe extern "C" fn()>; IDT_ENTRIES];
}

/// Registers pushed to the stack when handling an interrupt.
#[repr(C, packed)]
struct Registers { rsi: u64
                 , rdi: u64
                 , r11: u64
                 , r10: u64
                 , r9:  u64
                 , r8:  u64
                 , rdx: u64
                 , rcx: u64
                 , rax: u64
                 , int_id:  u32
                 , __pad_1: u32
                 , err_no:  u32
                 , __pad_2: u32
                 }

impl Registers {
    // Transform this struct into an array of u64s
    // (if you would ever want to do this)
    pub unsafe fn to_array(&self) -> [u64; 11] {
        [ self.rsi, self.rdi, self.r11, self.r10, self.r9
        , self.r8,  self.rdx, self.rcx, self.rax
        , mem::transmute([self.__pad_1, self.int_id])
        , mem::transmute([self.__pad_2, self.err_no])
        ]
    }
}

fn handle_exception(r: &Registers) {
    panic!( "ERROR {}: {}"
          , r.err_no
          , EXCEPTIONS[r.int_id as usize] );
}

/// An IDT entry is called a gate.
///
/// Based on code from the OS Dev Wiki
/// http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure
#[repr(C, packed)]
#[derive(Copy,Clone)]
struct Gate64 { /// bits 0 - 15 of the offset
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
              type_attr: u8
            , /// bits 16 - 31 of the offset
              offset_mid: u16
            , /// bits 32 - 63 of the offset
              offset_upper: u32
            , /// always zero (according to the spec, this is "reserved")
              reserved: u32
            }

impl Gate64 {
    /// Creates a new IDT gate marked as `absent`.
    ///
    /// This is basically just for filling the new IDT table
    /// with valid (but useless) gates upon init.
    const fn absent() -> Self {
        Gate64 { offset_lower: 0
               , selector: 0
               , zero: 0
               , type_attr: 0b0000_1110
               , offset_mid: 0
               , offset_upper: 0
               , reserved: 0
               }
    }
}

impl Gate for Gate64 {

    /// Creates a new IDT gate pointing at the given handler function.
    fn new(handler: Handler) -> Self {
        unsafe { // trust me on this.
                 // `mem::transmute()` is glorious black magic
            let (low, mid, high): (u16, u16, u32)
                = mem::transmute(handler);

            Gate64 { offset_lower: low
                   , selector: gdt64_offset
                   , zero: 0
                   , type_attr: 0b1000_1110
                   , offset_mid: mid
                   , offset_upper: high
                   , reserved: 0
                   }
        }
    }
}


struct Idt64([Gate64; IDT_ENTRIES]);

impl Idt for Idt64 {
    type Ptr = IdtPtr<Self>;

    /// Get the IDT pointer struct to pass to `lidt`
    fn get_ptr(&self) -> Self::Ptr {
        IdtPtr { limit: (mem::size_of::<Gate64>() * IDT_ENTRIES) as u16
               , base:  self as *const Idt64
               }
    }

    /// Add an entry for the given handler at the given index
    fn add_gate(&mut self, index: usize, handler: Handler) {
        self.0[index] = Gate64::new(handler)
    }

}

impl IdtPtrOps for IdtPtr<Idt64> {
    /// Load the IDT at the given location.
    /// This just calls `lidt`.
    unsafe fn load(&self) {
        asm!(  "lidt ($0)"
            :: "{rax}"(self)
            :: "volatile" );
    }
}

/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt64>
    = Mutex::new(Idt64([Gate64::absent(); IDT_ENTRIES]));
