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
use arch::cpu::dtable::DTable;

pub mod idt;
pub mod pics;
pub mod handlers;

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
            use $crate::arch::cpu::Registers;
            use $crate::arch::cpu::interrupts::handlers::ErrorCodeHandler;

            Registers::push();

            asm!( "mov rsi, [rsp + 9*8)]  // pop error code into rsi
                   mov rdi, rsp
                   add rdi, 10*8
                   sub rsp, 8   // align stack pointer

                   cli
                   call $0
                   sti

                   add rsp, 8   // un-align stack pointer"
                :: "s"($handler as ErrorCodeHandler)
                //  , "i"(size_of::<context::Registers>())
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
            use $crate::arch::cpu::Registers;
            use $crate::arch::cpu::interrupts::handlers::InterruptHandler;
            Registers::push();
            // Idt::disable_interrupts();

           asm!(  "mov rdi, rsp
                   add rdi, 9*8

                   cli
                   call $0
                   sti"
               :: "s"($handler as InterruptHandler)
                // , "i"(size_of::<context::Registers>())
               : "rdi" : "volatile", "intel");

            // Idt::enable_interrupts();
            Registers::pop();

            asm!("iretq" :::: "volatile", "intel");
            unreachable!();
       }
       isr
   }};
}

//==--------------------------------------------------------------------------==
// Top-level interrupt handling
lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        use self::handlers::*;

        // fill the IDT with empty ISRs so we don't throw faults
        // for i in 0..idt::ENTRIES {
        //     idt.add_handler(i, isr!() );
        // }

        idt .add_handler(0, isr!(interrupt: ex0))
            .add_handler(1, isr!(interrupt: ex1))
            .add_handler(2, isr!(interrupt: ex2))
            // ISR 3 reserved for breakpoints
            .add_handler(4, isr!(interrupt: ex4))
            .add_handler(5, isr!(interrupt: ex5))
            .add_handler(6, isr!(interrupt: ex6))
            .add_handler(7, isr!(interrupt: ex7))
            .add_handler(8, isr!(error: ex8))
             // ISR 9 is reserved in x86_64
            .add_handler(10, isr!(error: ex10))
            .add_handler(11, isr!(error: ex11))
            .add_handler(12, isr!(error: ex12))
            .add_handler(13, isr!(error: ex13))
            .add_handler(14, isr!(error: page_fault))
             // ISR 15: reserved
            .add_handler(16,  isr!(interrupt: ex16))
            .add_handler(17,  isr!(error: ex17))
            .add_handler(18,  isr!(interrupt: ex18))
            .add_handler(19,  isr!(interrupt: ex19))
            .add_handler(0x21, isr!(interrupt: keyboard))
            .add_handler(0xff, isr!(interrupt: test));

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
