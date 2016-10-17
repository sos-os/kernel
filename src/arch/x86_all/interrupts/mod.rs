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
        for i in 0..idt::ENTRIES {
            idt.add_handler(i, isr!(interrupt: empty_handler) );
        }

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
            .add_handler(0x20, isr!(interrupt: timer))
            .add_handler(0x21, isr!(interrupt: keyboard))
            .add_handler(0xff, isr!(interrupt: test));

        infoln!( dots: " . . ", "Adding interrupt handlers to IDT"
             , status: "[ OKAY ]");
        idt
    };
}


/// Initialize interrupt handling.
///
/// This function initializes the PICs, populates the IDT with interrupt
/// handlers, loads the IDT pointer, and enables interrupts.
///
/// This is called from the kernel during the init process.
pub unsafe fn initialize() {

    pics::initialize();
   // TODO: consider loading double-fault handler before anything else in case
   //       a double fault occurs during init?
    IDT.load();         // Load the IDT pointer

    debug!("Testing interrupt handling");
    asm!("int $0" :: "N" (0xff));

    Idt::enable_interrupts(); // enable interrupts

}
