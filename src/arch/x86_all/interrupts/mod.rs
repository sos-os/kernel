//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Intterupt handling on x86 machines.
//!
//! This module provides support for interrupt handling on both `x86` and
//! `x86_64` as a black box. Code that depends on this can use the same API
//! regardless of system word size.
use core::fmt;
use core::fmt::Write;

use arch::cpu::{control_regs, Registers};
use arch::cpu::dtable::DTable;

use io::term::CONSOLE;
use io::keyboard;

use vga::Color;

pub mod idt;
pub mod pics;

use self::idt::Idt;

/// State stored when handling an interrupt.
#[repr(C, packed)]
pub struct InterruptContext { /// callee-saved registers
                              pub registers: Registers
                            , /// interrupt ID number
                              pub int_id: usize
                            , _pad_1: u32
                            , /// error number
                              pub error_code:  u32
                            , _pad_2: u32
                            }

//impl InterruptContext {
//    /// Fetches the corresponding CPU exception for this interrupt, if this
//    /// interrupt is a CPU exception.
//    #[inline]
//
//}

#[derive(Debug)]
pub struct ExceptionInfo { pub name: &'static str
                         , pub mnemonic: &'static str
                         , pub irq_type: &'static str
                         , pub source: &'static str
                         }

/// x86 exceptions.
///
/// Taken from the list at
/// [http://wiki.osdev.org/Exceptions](http://wiki.osdev.org/Exceptions)
pub static EXCEPTIONS: [ExceptionInfo; 20]
    = [ ExceptionInfo { name: "Divide-By-Zero Error"
                      , mnemonic: "#DE", irq_type: "Fault"
                      , source: "DIV or IDIV instruction" }
      , ExceptionInfo { name: "RESERVED"
                      , mnemonic: "#DB", irq_type: "Fault/trap"
                      , source: "Reserved for Intel use" }
      , ExceptionInfo { name: "Non-Maskable Interrupt"
                      , mnemonic: "NMI", irq_type: "Interrupt"
                      , source: "Non-maskable external interrupt" }
      , ExceptionInfo { name: "Breakpoint"
                      , mnemonic: "#BP", irq_type: "Trap"
                      , source: "INT 3 instruction" }
      , ExceptionInfo { name: "Overflow"
                      , mnemonic: "#OF", irq_type: "Trap"
                      , source: "INTO instruction" }
      , ExceptionInfo { name: "BOUND Range Exceeded"
                      , mnemonic: "#BR", irq_type: "Fault"
                      , source: "BOUND instruction" }
      , ExceptionInfo { name: "Undefined Opcode"
                     , mnemonic: "#UD", irq_type: "Fault"
                     , source: "UD2 instruction or reserved opcode" }
      , ExceptionInfo { name: "Device Not Available"
                      , mnemonic: "#NM", irq_type: "Fault"
                      , source: "Floating-point or WAIT/FWAIT instruction \
                                 (no math coprocessor)" }
      , ExceptionInfo { name: "Double Fault"
                      , mnemonic: "#DF", irq_type: "Abort"
                      , source: "Any instruction that can generate an \
                                 exception, a NMI, or an INTR" }
      , ExceptionInfo { name: "Coprocessor Segment Overrun"
                      , mnemonic: "", irq_type: "Fault"
                      , source: "Any floating-point instruction" }
      , ExceptionInfo { name: "Invalid TSS"
                      , mnemonic: "#TS", irq_type: "Fault"
                      , source: "Task switch or TSS access" }
      , ExceptionInfo { name: "Segment Not Present"
                      , mnemonic: "#NP", irq_type: "Fault"
                      , source: "Loading segment registers or accessing\
                                 system segments" }
      , ExceptionInfo { name: "Stack-Segment Fault"
                      , mnemonic: "#SS", irq_type: "Fault"
                      , source: "Stack operations and SS register loads" }
      , ExceptionInfo { name: "General Protection"
                      , mnemonic: "#GP", irq_type: "Fault"
                      , source: "Any memory reference or other protection \
                                 checks" }
      , ExceptionInfo { name: "Page Fault"
                      , mnemonic: "#PF", irq_type: "Fault"
                      , source: "Any memory reference" }
      , ExceptionInfo { name: "RESERVED"
                      , mnemonic: "", irq_type: ""
                      , source: "RESERVED FOR INTEL USE \n This should never \
                                 happen. Something is very wrong." }
      , ExceptionInfo { name: "x87 FPU Floating-Point Error (Math Fault)"
                      , mnemonic: "#MF", irq_type: "Fault"
                      , source: "x87 FPU floating-point or WAIT/FWAIT\
                                 instruction" }
      , ExceptionInfo { name: "Alignment Check"
                      , mnemonic: "#AC", irq_type: "Fault"
                      , source: "Any data reference in memory" }
      , ExceptionInfo { name: "Machine Check"
                      , mnemonic: "#MC", irq_type: "Abort"
                      , source: "Model-dependent" }
      , ExceptionInfo { name: "SIMD Floating-Point Exception"
                      , mnemonic: "#XM", irq_type: "Fault"
                      , source: "SSE/SSE2/SSE3 floating-point instructions" }
       ];

/// Macro for making Interrupt Service Routines
macro_rules! isr {
   (exception $ex:expr, $name:ident, handler: $handler:ident) => {
       #[inline(never)] #[naked] #[no_mangle]
       pub unsafe extern fn $name() -> ! {
           asm!(  "push 0
                   push $0"
               :: "i"($ex)
               :: "volatile", "intel" );
           $crate::arch::cpu::Registers::push();
           asm!(  "mov rdi, rsp
                   call $0"
               :: "s"($handler as fn(&InterruptContext))
               :: "volatile", "intel");
           $crate::arch::cpu::Registers::pop();
           asm!( "add rsp, 16
                  iretq"
                :::: "volatile", "intel");
           unreachable!();

       }
   };
   (error $ex:expr, $name:ident, handler: $handler:ident) => {
       #[inline(never)] #[naked] #[no_mangle]
       pub unsafe extern fn $name() -> ! {
           asm!(  "push $0"
               :: "i"($ex)
               :: "volatile", "intel" );
           $crate::arch::cpu::Registers::push();
           asm!(  "mov rdi, rsp
                   call $0"
               :: "s"($handler as fn(&InterruptContext))
               :: "volatile", "intel");
           $crate::arch::cpu::Registers::pop();
           asm!( "add rsp, 16
                  iretq"
                :::: "volatile", "intel");
           unreachable!();
       }
   };
   (interrupt $id:expr, $name:ident, handler: $handler:ident) => {
       #[inline(never)] #[naked] #[no_mangle]
       pub unsafe extern fn $name() -> ! {
           asm!(  "push 0
                   push $0"
               :: "i"($id)
               :: "volatile", "intel" );
           $crate::arch::cpu::Registers::push();
           asm!(  "mov rdi, rsp
                   call $0"
               :: "s"($handler as fn(&InterruptContext))
               :: "volatile", "intel");
           $crate::arch::cpu::Registers::pop();
           asm!( "add rsp, 16
                  iretq"
                :::: "volatile", "intel");
           unreachable!();
       }
   };
   (interrupt $id:expr, $name:ident) => {
       isr! { interrupt $id, $name, handler: handle_interrupt }
   };
   (error $id:expr, $name:ident) => {
       isr! { error $id, $name, handler: handle_cpu_exception }
   };
   (exception $id:expr, $name:ident) => {
       isr! { exception $id, $name, handler: handle_cpu_exception }
   };
}
//==--------------------------------------------------------------------------==
// Top-level interrupt handling
lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        // fill the IDT with empty ISRs so we don't throw faults
        for i in 0..idt::ENTRIES {
            idt.add_handler(i, empty_isr);
        }

        idt.add_handler(0,   isr_0)
        .add_handler(1,   isr_1)
        .add_handler(2,   isr_2)
        .add_handler(3,   isr_3)
        .add_handler(4,   isr_4)
        .add_handler(5,   isr_5)
        .add_handler(6,   isr_6)
        .add_handler(7,   isr_7)
        .add_handler(8,   isr_8)
         // ISR 9 is reserved in x86_64
        .add_handler(10,  isr_10)
        .add_handler(11,  isr_11)
        .add_handler(12,  isr_12)
        .add_handler(13,  isr_13)
        .add_handler(14,  isr_14_pagefault)
         // ISR 15: reserved
        .add_handler(16,  isr_16)
        .add_handler(17,  isr_17)
        .add_handler(18,  isr_18)
        .add_handler(19,  isr_19)
        .add_handler(0x21, keyboard_isr)
        .add_handler(0x80, test_isr);

        println!("{:<38}{:>40}", " . . Adding interrupt handlers to IDT"
             , "[ OKAY ]");
        idt
    };
}

/// Kernel interrupt-handling function.
///
/// Assembly ISRs call into this, and it dispatches interrupts to
/// the appropriate consumers.
//  TODO: should this be #[cold]?
#[no_mangle] #[inline(never)]
pub fn handle_interrupt(state: &InterruptContext) {
    let id = state.int_id;
    match id {
        // System timer
        0x20 => { /* TODO: make this work
                    TODO: should this IRQ get its own handler?
                  */
                 println!("timer!");
                 }
        // Loonix syscall vector
      , 0x80 => { // TODO: currently, we do nothing here, do we want
                  // our syscalls on this vector as well?
        }
      , _ =>  {
          // unknown interrupt. silently do nothing?
          println!("Unknown interrupt: #{} Sorry!", id);
      }
    }
    // send the PICs the end interrupt signal
    unsafe {
        pics::end_pic_interrupt(id as u8);
        Idt::enable_interrupts();
    }
}

#[no_mangle] #[inline(never)]
pub fn keyboard_handler(state: &InterruptContext) {
    println!("keyboard happened");
    if let Some(input) = keyboard::read_char() {
        if input == '\r' {
            println!("");
        } else {
            print!("{}", input);
        }
    }
    // send the PICs the end interrupt signal
    unsafe {
        pics::end_pic_interrupt(state.int_id as u8);
        Idt::enable_interrupts();
    }
}

/// Handle a CPU exception with a given interrupt context.
//  TODO: should this be #[cold]?
#[no_mangle] #[inline(never)]
pub fn handle_cpu_exception(state: &InterruptContext) {
    let id = state.int_id;
    let ex_info = &EXCEPTIONS[id];
    let cr_state = control_regs::dump();
    let _ = write!( CONSOLE.lock()
                          .set_colors(Color::White, Color::Blue)
                        //   .clear()
                  , "CPU EXCEPTION {}: {}\n\
                     {} on vector {} with error code {:#x}\n\
                     Source: {}.\nThis is fine.\n\n"
                  , ex_info.mnemonic, ex_info.name, ex_info.irq_type
                  , id, state.error_code
                  , ex_info.source );

    // TODO: parse error codes
    let _ = match id {
        _ => write!( CONSOLE.lock()
                             .set_colors(Color::White, Color::Blue)
                    , "Registers:\n{:?}\n    {}\n"
                    , state.registers
                    , cr_state
                    )
    };
    // TODO: stack dumps please

    loop { }
}

/// Handles page fault exceptions
#[no_mangle] #[inline(never)]
pub fn handle_page_fault(state: &InterruptContext) {
    let _ = write!( CONSOLE.lock()
                           .set_colors(Color::White, Color::Blue)
                        //   .clear()
                  , "PAGE FAULT EXCEPTION\nCode: {:#x}\n\n{}"
                  , state.error_code
                  , PageFaultErrorCode::from_bits_truncate(state.error_code)
                  );
    // TODO: stack dumps please

    loop { }
}

#[no_mangle] #[inline(never)]
pub fn test_handler(state: &InterruptContext) {
    assert_eq!(state.int_id, 0x80);
    println!("{:>47}", "[ OKAY ]");
    // send the PICs the end interrupt signal
    unsafe {
        pics::end_pic_interrupt(state.int_id as u8);
        Idt::enable_interrupts();
    }
}

isr! { exception 0, isr_0 }
isr! { exception 1, isr_1 }
isr! { exception 2, isr_2 }
isr! { exception 3, isr_3 }
isr! { exception 4, isr_4 }
isr! { exception 5, isr_5 }
isr! { exception 6, isr_6 }
isr! { exception 7, isr_7 }
isr! { error 8, isr_8 }
 // ISR 9 is reserved in x86_64
isr! { error 10, isr_10 }
isr! { error 11, isr_11 }
isr! { error 12, isr_12 }
isr! { error 13, isr_13 }
isr! { error 14, isr_14_pagefault, handler: handle_page_fault }
 // ISR 15: reserved
isr! { exception 16, isr_16 }
isr! { error 17, isr_17 }
isr! { exception 18, isr_18 }
isr! { exception 19, isr_19 }
isr! { interrupt 0x21, keyboard_isr, handler: keyboard_handler }
isr! { interrupt 0x80, test_isr, handler: test_handler }

isr!{ interrupt 255, empty_isr }


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
   // TODO: consider loading double-fault handler before anything else in case
   //       a double fault occurs during init?
    IDT.load();         // Load the IDT pointer

    print!(" . . Testing interrupt handling");
    asm!("int $0" :: "N" (0x80));

    Idt::enable_interrupts(); // enable interrupts

}

bitflags! {
    flags PageFaultErrorCode: u32 {
        /// If 1, the error was caused by a page that was present.
        /// Otherwise, the page was non-present.
        const PRESENT = 1 << 0
      , /// If 1, the error was caused by a read. If 0, the cause was a write.
        const READ_WRITE = 1 << 1
      , /// If 1, the error was caused during user-mode execution.
        /// If 0, the processor was in kernel mode.
        const USER_MODE = 1 << 2
      , /// If 1, the fault was caused by reserved bits set to 1 during a fetch.
        const RESERVED = 1 << 3
      , /// If 1, the fault was caused during an instruction fetch.
        const INST_FETCH = 1 << 4
      , /// If 1, there was a protection key violation.
        const PROTECTION = 1 << 5
    }
}

impl fmt::Display for PageFaultErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f, "Caused by {}{}{} during a {}{} executing in {} mode."
              , if self.contains(PRESENT) { "a present page" }
                else { "a non-present page" }
              , if self.contains(PROTECTION) { " protection-key violation" }
                else { "" }
              , if self.contains(RESERVED) { " reserved bits set to one "}
                else { "" }
              , if self.contains(READ_WRITE) { "read" } else { "write" }
              , if self.contains(INST_FETCH) { " in an instruction fetch"}
                else { "" }
              , if self.contains(USER_MODE) { "user" } else { "kernel" }            )
    }
}
