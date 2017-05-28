//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use cpu::interrupts::pics;
use cpu::interrupts::idt::{Gate, Idt};

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
// TODO: make the result returned by this meaningful?
#[inline]
pub unsafe fn initialize() -> Result<(), ()>{

    pics::initialize();
   // TODO: consider loading double-fault handler before anything else in case
   //       a double fault occurs during init?
    IDT.load();         // Load the IDT pointer
    //
    // debug!("Testing interrupt handling");
    // asm!("int $0" :: "N" (0xff));

    Idt::enable_interrupts(); // enable interrupts
    Ok(())

}

macro_rules! exception_inner {
    ($title:expr, $kind:expr, $source:expr, $f:expr) => {
        use vga::{CONSOLE, Color};
        use core::fmt::Write;
        let _ = write!( CONSOLE.lock()
                               .set_colors(Color::White, Color::Blue)
                      , "EVERYTHING IS FINE: {}{} at {:p}\n\
                         Source: {}.\nThis is fine.\n\n\
                         {:?}"
                         , $title, $kind
                         , (*$f).rip
                         , $source
                         , *$f);
    };
    ($title:expr, $kind:expr, $source:expr, $f:expr, $e:expr) => {
        use vga::{CONSOLE, Color};
        use core::fmt::Write;
        let _ = write!( CONSOLE.lock()
                               .set_colors(Color::White, Color::Blue)
                      , "EVERYTHING IS FINE: {}{} at {:p}\n\
                         Source: {}.\n
                         Error code: {:x}\nThis is fine.\n\n\
                         {:?}"
                         , $title, $kind
                         , (*$f).rip
                         , $source
                         , $e
                         , *$f);
    };
}

macro_rules! exceptions {
    ( fault: $name:ident, $title:expr, $source:expr, $($tail:tt)* ) => {
        #[doc=$title]
        extern "x86-interrupt" fn $name(frame: &InterruptFrame) {
            exception_inner! ($title, "Fault", $source, frame);
            loop {}
        }

        exceptions! {  $($tail)* }
    };
     ( fault (code): $name:ident, $title:expr, $source:expr, $($tail:tt)* ) => {
        #[doc=$title]
        extern "x86-interrupt" fn $name( frame: &InterruptFrame
                                       , error_code: usize) {
           exception_inner! ($title, "Fault", $source, frame, error_code);
           loop {}
       }
       exceptions! { $($tail)* }
   };
     ( trap: $name:ident, $title:expr, $source:expr, $($tail:tt)* ) => {
         #[doc=$title]
         extern "x86-interrupt" fn $name(frame: &InterruptFrame) {
             exception_inner! ($title, "Trap", $source, frame);
         }

         exceptions! { $($tail)* }
     };
      ( ) => {};

}
exceptions! {
    fault: divide_by_zero, "Divide by Zero Error",
           "DIV or IDIV instruction",
    fault: nmi, "Non-Maskable Interrupt",
          "Non-maskable external interrupt",
    trap: overflow, "Overflow", "INTO instruction",
    fault: bound_exceeded, "BOUND range exceeded",
          "BOUND instruction",
    fault: undefined_opcode, "Undefined Opcode",
           "UD2 instruction or reserved opcode",
    fault: device_not_available, "Device Not Available"
         , "Floating-point or WAIT/FWAIT instruction \
            (no math coprocessor)",
    fault (code): double_fault, "Double Fault"
         , "Any instruction that can generate an exception, a NMI, or \
            an INTR",
    fault (code): invalid_tss, "Invalid TSS"
         , "Task switch or TSS access",
    fault (code): segment_not_present, "Segment Not Present"
         , "Loading segment registers or accessing \
            system segments",
    fault (code): general_protection_fault, "General Protection Fault"
         , "Any memory reference or other protection checks",
    fault (code): stack_segment_fault, "Stack Segment Fault"
         , "Stack operations and SS register loads",
    fault: floating_point_error
         , "x87 FPU Floating-Point Error (Math Fault)"
         , "x87 FPU floating-point or WAIT/FWAIT instruction",
    fault: machine_check, "Machine Check"
         , "Model-dependent (probably hardware!)",
    fault (code): alignment_check, "Alignment Check"
         , "Any data reference in memory",
    fault: simd_fp_exception, "SIMD Floating-Point Exception"
         , "SSE/SSE2/SSE3 floating-point instructions",
}

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        use cpu::interrupts::*;

        // TODO: log each handler as it's added to the IDT? that way we can
        //       trace faults occurring during IDT population (if any)
        //          - eliza, 5/22/2017
        idt.divide_by_zero = Gate::from(divide_by_zero as InterruptHandler);
        idt.nmi = Gate::from(nmi as InterruptHandler);
        idt.overflow = Gate::from(overflow as InterruptHandler);
        idt.overflow.set_trap();
        idt.bound_exceeded = Gate::from(bound_exceeded as InterruptHandler);
        idt.undefined_opcode = Gate::from(undefined_opcode as InterruptHandler);
        idt.device_not_available = Gate::from(device_not_available as InterruptHandler);
        idt.double_fault = Gate::from(double_fault as ErrorCodeHandler);
        idt.invalid_tss = Gate::from(invalid_tss as ErrorCodeHandler);
        idt.segment_not_present = Gate::from(segment_not_present as ErrorCodeHandler);
        idt.stack_segment_fault = Gate::from(stack_segment_fault as ErrorCodeHandler);
        idt.general_protection_fault = Gate::from(general_protection_fault as ErrorCodeHandler);
        idt.page_fault = Gate::from(page_fault as ErrorCodeHandler);

        idt.floating_point_error = Gate::from(floating_point_error as InterruptHandler);
        idt.alignment_check = Gate::from(alignment_check as ErrorCodeHandler);
        idt.machine_check = Gate::from(machine_check as InterruptHandler);
        idt.simd_fp_exception = Gate::from(simd_fp_exception as InterruptHandler);

        idt.breakpoint = Gate::from(breakpoint as InterruptHandler);
        idt.page_fault = Gate::from(page_fault as ErrorCodeHandler);

        idt.interrupts[0x20 - 32] = Gate::from(timer as InterruptHandler);
        idt.interrupts[0x21 - 32] = Gate::from(keyboard as InterruptHandler);
        idt.interrupts[0xff - 32] = Gate::from(test as InterruptHandler);

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
       pics::end_pic_interrupt(0x03);
   }
}

/// Empty dummy handler for undefined interrupts.
#[no_mangle] #[inline(never)]
pub extern "x86-interrupt" fn empty_handler(_frame: &InterruptFrame) {
    // TODO: it would be nice to know *which vector* the dummy interrupt
    //      fired on, for debugging purposes...
    //          - eliza, 05/25/2017
    debug!("an empty interrupt fired!")
}
