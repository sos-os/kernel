//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Intterupt handling on x86 machines.
//!
//! This module provides support for interrupt handling on both `x86` and
//! `x86_64` as a black box. Code that depends on this can use the same API
//! regardless of system word size.
#![warn(missing_docs)]
pub mod idt;
pub mod pics;

use vga::{CONSOLE, Color};

use core::fmt;
use core::fmt::Write;

use context::InterruptFrame;

/// Number of interrupt vectors corresponding to CPU exceptions.
///
/// These are the first 32 vectors in the IDT.
pub const NUM_EXCEPTIONS: usize = 32;

/// An ISR that handles a regular interrupt
pub type InterruptHandler = extern "x86-interrupt" fn (&InterruptFrame);
/// An ISR that handles an error with an error code
pub type ErrorCodeHandler = extern "x86-interrupt" fn (&InterruptFrame, usize);

/// A description of a CPU exception
#[derive(Debug)]
pub struct ExceptionInfo { /// The name of the exception
                           pub name: &'static str
                         , /// The mnemomic code for the exception
                           pub mnemonic: &'static str
                         , /// The type of IRQ for this exception
                           /// - fault
                           /// - trap
                           /// - interrupt
                           pub irq_type: &'static str
                         , /// The source triggering the exception.
                           ///
                           /// Typically this refers to what opcode(s) can
                           /// cause this exception.
                           pub source: &'static str
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

/// Handler for the system timer interrupt
pub extern "x86-interrupt" fn timer(_frame: &InterruptFrame) {
    // do nothing, just signal the pics to end the IRQ
    // println!("timer!");
    unsafe { pics::end_pic_interrupt(0x21); }
}



/// Handles page fault exceptions
#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn page_fault( frame: &InterruptFrame, error_code: usize) {
   let _ = write!( CONSOLE.lock()
                      .set_colors(Color::White, Color::Blue)
                   //   .clear()
             , "IT'S NOT MY FAULT: Page Fault at {:p} \
                \nError code: {:#x}\n\n{}\n{:?}"
             , (*frame).rip
             , error_code
             , PageFaultErrorCode::from_bits_truncate(error_code as u32)
             , *frame
             );
   // TODO: stack dumps please

   loop { }
}

/// Test interrupt handler for ensuring that the IDT is configured correctly.
#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn test(_frame: &InterruptFrame) {
   // assert_eq!(state.int_id, 0x80);
   kinfoln!(dots: " . . ", target: "Testing interrupt handling:", "[ OKAY ]");
   // send the PICs the end interrupt signal
   unsafe {
       pics::end_pic_interrupt(0xff);
   }
}

/// Empty dummy handler for undefined interrupts.
#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn empty_handler(_frame: &InterruptFrame) {
   // assert_eq!(state.int_id, 0x80);
   println!("interrupt");
   // send the PICs the end interrupt signal
   // unsafe {
   //     end_pic_interrupt(0xff);
   // }
}
