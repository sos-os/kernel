//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use cpu::interrupts::{idt, pics};
use cpu::interrupts::idt::Idt;

use cpu::context::InterruptFrame;
use cpu::dtable::DTable;

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
        use cpu::interrupts::handlers::*;
        use core::mem::transmute;

        // fill the IDT with empty ISRs so we don't throw faults
        for i in 0..idt::ENTRIES {
            idt.add_handler(i, empty_handler as idt::Handler);
        }
        unsafe {
            // TODO: we can do this in a type-safe manner...
        idt .add_handler(0, transmute(ex0))
            .add_handler(1, transmute(ex1))
            .add_handler(2, transmute(ex2))
            // ISR 3 reserved for breakpoints
            .add_handler(4, transmute(ex4))
            .add_handler(5, transmute(ex5))
            .add_handler(6, transmute(ex6))
            .add_handler(7, transmute(ex7))
            .add_handler(8, transmute(ex8))
             // ISR 9 is reserved in x86_64
            .add_handler(10, transmute(ex10))
            .add_handler(11, transmute(ex11))
            .add_handler(12, transmute(ex12))
            .add_handler(13, transmute(ex13))
            .add_handler(14, transmute(page_fault))
             // ISR 15: reserved
            .add_handler(16,  transmute(ex16))
            .add_handler(17,  transmute(ex17))
            .add_handler(18,  transmute(ex18))
            .add_handler(19,  transmute(ex19))
            .add_handler(0x20, transmute(timer))
            .add_handler(0x21, transmute(keyboard))
            .add_handler(0xff, transmute(test));
        }


        kinfoln!( dots: " . . ", target: "Adding interrupt handlers to IDT"
                , "[ OKAY ]");
        idt
    };
}


#[no_mangle] #[inline(never)]
pub extern "C" fn keyboard(_frame: *const InterruptFrame) {
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
