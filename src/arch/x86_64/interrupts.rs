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
