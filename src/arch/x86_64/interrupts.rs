//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use cpu::interrupts::pics;
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
        use cpu::interrupts::*;
        // TODO: use semantic names for handlers & idt field refs
        //       (i was too lazy to look them up while porting this to the new
        //        IDT api)
        //          - eliza, 5/22/2017

        // TODO: log each handler as it's added to the IDT? that way we can
        //       trace faults occurring during IDT population (if any)
        //          - eliza, 5/22/2017

        // i don't know why these all need to be cast? very weird. should fix.
        idt.divide_by_zero.set_handler(ex0 as InterruptHandler);
        idt.debug.set_handler(ex1 as InterruptHandler);
        idt.nmi.set_handler(ex2 as InterruptHandler);

        idt.overflow.set_handler(ex4 as InterruptHandler);
        idt.bound_exceeded.set_handler(ex5 as InterruptHandler);
        idt.undefined_opcode.set_handler(ex6 as InterruptHandler);
        idt.device_not_available.set_handler(ex7 as InterruptHandler);
        idt.double_fault.set_handler(ex8 as ErrorCodeHandler);

        idt.invalid_tss.set_handler(ex10 as ErrorCodeHandler);
        idt.segment_not_present.set_handler(ex11 as ErrorCodeHandler);
        idt.stack_segment_fault.set_handler(ex12 as ErrorCodeHandler);
        idt.general_protection_fault.set_handler(ex13 as ErrorCodeHandler);
        idt.page_fault.set_handler(page_fault as ErrorCodeHandler);
        idt.floating_point_error.set_handler(ex16 as InterruptHandler);
        idt.alignment_check.set_handler(ex17 as ErrorCodeHandler);
        idt.machine_check.set_handler(ex18 as InterruptHandler);
        idt.simd_fp_exception.set_handler(ex19 as InterruptHandler);
        idt[0x20].set_handler(timer as InterruptHandler);
        idt[0x21].set_handler(keyboard as InterruptHandler);
        idt[0xff].set_handler(test as InterruptHandler);

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

#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn breakpoint(frame: &InterruptFrame) {
    println!("Breakpoint! Frame: {:#?}", frame);
   // send the PICs the end interrupt signal
   unsafe {
       pics::end_pic_interrupt(0x21);
   }
}
