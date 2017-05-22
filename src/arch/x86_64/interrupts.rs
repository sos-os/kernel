//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use cpu::interrupts::{idt, pics};
use cpu::interrupts::idt::{Idt, Gate};

use cpu::context::InterruptFrame;
use cpu::dtable::DTable;

use core::convert::From;

//==--------------------------------------------------------------------------==
// Top-level interrupt handling

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

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        use cpu::interrupts::*;
        use core::mem::transmute;

        // fill the IDT with empty ISRs so we don't throw faults
        for i in 0..idt::ENTRIES {
            idt.add_handler(i, empty_handler as InterruptHandler);
        }
        unsafe {
            // TODO: we can do this in a type-safe manner...
        idt .add_handler(0, ex0 as InterruptHandler)
            .add_handler(1, ex1 as InterruptHandler)
            .add_handler(2, ex2 as InterruptHandler)
            // ISR 3 reserved for breakpoints
            .add_handler(4, ex4 as InterruptHandler)
            .add_handler(5, ex5 as InterruptHandler)
            .add_handler(6, ex6 as InterruptHandler)
            .add_handler(7, ex7 as InterruptHandler)
            .add_handler(8, ex8 as ErrorCodeHandler)
             // ISR 9 is reserved in x86_64
            .add_handler(10, ex10 as ErrorCodeHandler)
            .add_handler(11, ex11 as ErrorCodeHandler)
            .add_handler(12, ex12 as ErrorCodeHandler)
            .add_handler(13, ex13 as ErrorCodeHandler)
            .add_handler(14, page_fault as ErrorCodeHandler)
             // ISR 15: reserved
            .add_handler(16,  ex16 as InterruptHandler)
            .add_handler(17,  ex17 as ErrorCodeHandler)
            .add_handler(18,  ex18 as InterruptHandler)
            .add_handler(19,  ex19 as InterruptHandler)
            .add_handler(0x20, timer as InterruptHandler)
            .add_handler(0x21, keyboard as InterruptHandler)
            .add_handler(0xff, test as InterruptHandler);
        }


        kinfoln!( dots: " . . ", target: "Adding interrupt handlers to IDT"
                , "[ OKAY ]");
        idt
    };
}


#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn keyboard(_frame: &InterruptFrame) {
    use io::keyboard;

    // println!("keyboard happened");
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
