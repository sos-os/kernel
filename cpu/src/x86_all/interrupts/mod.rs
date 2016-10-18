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

pub mod idt;
pub mod pics;
pub mod handlers;

/// Macro for making Interrupt Service Routines
#[macro_export]
macro_rules! isr {
   (error: $handler:ident) => {{
       #[inline(never)] #[naked]
       unsafe extern "C" fn isr() -> ! {
            use $crate::Registers;
            use $crate::interrupts::handlers::ErrorCodeHandler;

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
            use $crate::Registers;
            use $crate::interrupts::handlers::InterruptHandler;

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
