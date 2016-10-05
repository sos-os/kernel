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
use arch::cpu::context::InterruptFrame;

use io::term::CONSOLE;
use io::keyboard;

use vga::Color;

pub mod idt;
pub mod pics;

use self::idt::Idt;


/// State stored when handling an interrupt.
#[repr(C, packed)]
// pub struct InterruptContext { /// callee-saved registers
//                               pub registers: Registers
//                             , /// interrupt ID number
//                               pub int_id: usize
//                             , _pad_1: u32
//                             , /// error number
//                               pub error_code:  u32
//                             , _pad_2: u32
//                             }


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
   (error: $handler:ident) => {{
       #[inline(never)] #[naked]
       unsafe extern "C" fn isr() -> ! {
            use $crate::arch::cpu::{Registers, context};
            use core::mem::size_of;

            Registers::push();

            asm!( "mov rsi, [rsp + ($0 * 8)]  // pop error code into rsi
                   mov rdi, rsp
                   add rdi, ($0 + 1) * 8
                   sub rsp, 8   // align stack pointer

                   cli
                   call $1
                   sti

                   add rsp, 8   // un-align stack pointer"
                :: "i"(size_of::<Registers>())
                 , "s"($handler as extern "C" fn( *const context::InterruptFrame
                                                , u64))
                : "rsi", "rdi"
                : "volatile", "intel");

            Registers::pop();

            asm!( "add rsp, 8     // remove error code from stack
                   iretq" :::: "volatile", "intel");
            unreachable!();
       }
       isr
   }};
   (interrupt: $handler:ident) => {{
       #[inline(never)] #[naked]
       unsafe extern "C" fn isr() -> ! {
            use $crate::arch::cpu::{Registers, context};
            use core::mem::size_of;

            Registers::push();
            // Idt::disable_interrupts();

           asm!(  "mov rdi, rsp
                   add rdi, $0

                   cli
                   call $1
                   sti"
               :: "i"(size_of::<Registers>() * 8)
                , "s"($handler as extern "C" fn(*const context::InterruptFrame))
               : "rdi" : "volatile", "intel");

            // Idt::enable_interrupts();
            Registers::pop();

            asm!("iretq" :::: "volatile", "intel");
            unreachable!();
       }
       isr
   }};
}

macro_rules! make_handlers {
    ( $(ex $ex_num:expr, $name:ident),+ ) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name(frame: *const InterruptFrame) {
                unsafe {
                    let ex_info = &EXCEPTIONS[$ex_num];
                    // let cr_state = control_regs::dump();
                    let _ = write!( CONSOLE.lock()
                                           .set_colors(Color::White, Color::Blue)
                                  , "{} EXCEPTION: {}\n\
                                     {} on vector {}\n\
                                     Source: {}.\nThis is fine.\n\n\
                                     {:?}"
                                     , ex_info.name, ex_info.mnemonic
                                     , ex_info.irq_type
                                     , $ex_num
                                     , ex_info.source
                                     , *frame);
                    loop { }
                }
            }
        )+
    };
    ( $(err $ex_num:expr, $name:ident) ,+ ) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name( frame: *const InterruptFrame
                                   , err_code: u64) {
                unsafe {
                    let ex_info = &EXCEPTIONS[$ex_num];
                    // let cr_state = control_regs::dump();
                    let _ = write!( CONSOLE.lock()
                                           .set_colors(Color::White, Color::Blue)
                                  , "{} EXCEPTION: {}\n\
                                     {} on vector {} with error code {:#x}\n\
                                     Source: {}.\nThis is fine.\n\n\
                                     {:?}"
                                  , ex_info.name, ex_info.mnemonic
                                  , ex_info.irq_type
                                  , $ex_num, err_code
                                  , ex_info.source
                                  , *frame);
                    loop { }
                }
            }
        )+
    };
}
make_handlers! { ex 0, ex0_handler
               , ex 1, ex1_handler
               , ex 2, ex2_handler
                // ex 3 is breakpoint
               , ex 4, ex4_handler
               , ex 5, ex5_handler
               , ex 6, ex6_handler
               , ex 7, ex7_handler
               , ex 16, ex16_handler
               , ex 18, ex18_handler
               , ex 19, ex19_handler
               }
make_handlers! { err 8, ex8_handler
               , err 10, ex10_handler
               , err 11, ex11_handler
               , err 12, ex12_handler
               , err 13, ex13_handler
               , err 17, ex17_handler }


//==--------------------------------------------------------------------------==
// Top-level interrupt handling
lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        // fill the IDT with empty ISRs so we don't throw faults
        // for i in 0..idt::ENTRIES {
        //     idt.add_handler(i, isr!() );
        // }

        idt .add_handler(0, isr!(interrupt: ex0_handler))
            .add_handler(1, isr!(interrupt: ex1_handler))
            .add_handler(2, isr!(interrupt: ex2_handler))
            // ISR 3 reserved for breakpoints
            .add_handler(4, isr!(interrupt: ex4_handler))
            .add_handler(5, isr!(interrupt: ex5_handler))
            .add_handler(6, isr!(interrupt: ex6_handler))
            .add_handler(7, isr!(interrupt: ex7_handler))
            .add_handler(8, isr!(error: ex8_handler))
             // ISR 9 is reserved in x86_64
            .add_handler(10, isr!(error: ex10_handler))
            .add_handler(11, isr!(error: ex11_handler))
            .add_handler(12, isr!(error: ex12_handler))
            .add_handler(13, isr!(error: ex13_handler))
            .add_handler(14, isr!(error: page_fault_handler))
             // ISR 15: reserved
            .add_handler(16,  isr!(interrupt: ex16_handler))
            .add_handler(17,  isr!(error: ex17_handler))
            .add_handler(18,  isr!(interrupt: ex18_handler))
            .add_handler(19,  isr!(interrupt: ex19_handler))
            .add_handler(0x21, isr!(interrupt: keyboard_handler))
            .add_handler(0xff, isr!(interrupt: test_handler));

        println!("{:<38}{:>40}", " . . Adding interrupt handlers to IDT"
             , "[ OKAY ]");
        idt
    };
}

// /// Kernel interrupt-handling function.
// ///
// /// Assembly ISRs call into this, and it dispatches interrupts to
// /// the appropriate consumers.
// //  TODO: should this be #[cold]?
// #[no_mangle] #[inline(never)]
// pub fn handle_interrupt(state: &InterruptFrame) {
//     let id = state.int_id;
//     match id {
//         // System timer
//         0x20 => { /* TODO: make this work
//                     TODO: should this IRQ get its own handler?
//                   */
//                  println!("timer!");
//                  }
//         // Loonix syscall vector
//       , 0x80 => { // TODO: currently, we do nothing here, do we want
//                   // our syscalls on this vector as well?
//         }
//       , _ =>  {
//           // unknown interrupt. silently do nothing?
//           println!("Unknown interrupt: #{} Sorry!", id);
//       }
//     }
//     // send the PICs the end interrupt signal
//     unsafe {
//         pics::end_pic_interrupt(id as u8);
//         Idt::enable_interrupts();
//     }
// }

#[no_mangle] #[inline(never)]
pub extern "C" fn keyboard_handler(state: *const InterruptFrame) {
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
        pics::end_pic_interrupt(0x21);
    }
}


/// Handles page fault exceptions
#[no_mangle] #[inline(never)]
pub extern "C" fn page_fault_handler( state: *const InterruptFrame
                                    , error_code: u64) {
    let _ = write!( CONSOLE.lock()
                           .set_colors(Color::White, Color::Blue)
                        //   .clear()
                  , "PAGE FAULT EXCEPTION\nCode: {:#x}\n\n{}"
                  , error_code
                  , PageFaultErrorCode::from_bits_truncate(error_code as u32)
                  );
    // TODO: stack dumps please

    loop { }
}

#[no_mangle] #[inline(never)]
pub extern "C" fn test_handler(state: *const InterruptFrame) {
    // assert_eq!(state.int_id, 0x80);
    println!("{:>47}", "[ OKAY ]");
    // send the PICs the end interrupt signal
    unsafe {
        pics::end_pic_interrupt(0xff);
    }
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
   // TODO: consider loading double-fault handler before anything else in case
   //       a double fault occurs during init?
    IDT.load();         // Load the IDT pointer

    print!(" . . Testing interrupt handling");
    asm!("int $0" :: "N" (0xff));

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
