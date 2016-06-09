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

use io::term::CONSOLE;
use io::keyboard;

use vga::Color;

use spin::Mutex;

pub mod idt;
pub mod pics;

use self::idt::Idt;

/// State stored when handling an interrupt.
#[repr(C, packed)]
pub struct InterruptContext { /// callee-saved registers
                              pub registers: Registers
                            , /// interrupt ID number
                              pub int_id: usize
                            //, _pad_1: u32
                            //, /// error number
                            //  pub err_no:  u32
                            //, _pad_2: u32
                            }

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
                      , source: "Floating-point or WAIT/FWAIT instruction\
                                 (no math coprocessor)" }
      , ExceptionInfo { name: "Double Fault"
                      , mnemonic: "#DF", irq_type: "Abort"
                      , source: "Any instruction that can generate an\
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
                      , source: "Any memory reference or other protection\
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

//==--------------------------------------------------------------------------==
// Top-level interrupt handling
/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt> = Mutex::new(Idt::new());

macro_rules! isr {
    (exception $ex:expr, $name:ident, handler: $handler:ident) => {{
        #[inline(never)] #[naked] #[no_mangle]
        pub unsafe extern "C" fn $name() {
            asm!(  "push 0
                    push $0"
                :: "i"($ex)
                :: "volatile", "intel" );
            $crate::arch::cpu::Registers::push();
            asm!( concat!(
                    "mov rdi, rsp
                     call ", stringify!($handler))
                :: :: "volatile", "intel");
            $crate::arch::cpu::Registers::pop();
            asm!( "add rsp, 16
                   iretq"
                 :::: "volatile", "intel");
            unreachable!();

        }
        $crate::arch::cpu::interrupts::idt::Gate::from(
            $name as $crate::arch::cpu::interrupts::idt::Handler)
    }};
    (error $ex:expr, $name:ident, handler: $handler:ident) => {{
        #[inline(never)] #[naked] #[no_mangle]
        pub unsafe extern "C" fn $name() -> ! {
            asm!(  "push $0"
                :: "i"($ex)
                :: "volatile", "intel" );
            $crate::arch::cpu::Registers::push();
            asm!( concat!(
                    "mov rdi, rsp
                     call ", stringify!($handler))
                :: :: "volatile", "intel");
            $crate::arch::cpu::Registers::pop();
            asm!( "add rsp, 16
                   iretq"
                 :::: "volatile", "intel");
            unreachable!();
            $crate::arch::cpu::interrupts::idt::Gate::from(
                $name as $crate::arch::cpu::interrupts::idt::Handler)
        }
    }};
    (interrupt $id:expr, $name:ident, handler: $handler:ident) => {{
        #[inline(never)] #[naked] #[no_mangle]
        pub unsafe extern "C" fn $name() -> ! {
            asm!(  "push $0"
                :: "i"($id)
                :: "volatile", "intel" );
            $crate::arch::cpu::Registers::push();
            asm!( concat!(
                    "mov rdi, rsp
                     call ", stringify!($handler))
                :: :: "volatile", "intel");
            $crate::arch::cpu::Registers::pop();
            asm!( "add rsp, 8
                   iretq"
                 :::: "volatile", "intel");
            unreachable!();
        }
        $crate::arch::cpu::interrupts::idt::Gate::from(
            $name as $crate::arch::cpu::interrupts::idt::Handler)
    }};
    (interrupt $id:expr, $name:ident) => {
        isr! { interrupt $id, $name, handler: handle_interrupt }
    };
    (error $id:expr, $name:ident) => {
        isr! { error $id, $name, handler: handle_cpu_exception }
    };
    (exception $id:expr, $name:ident) => {
        isr! { exception $id, $name, handler: handle_cpu_exception }
    };
}

/// Kernel interrupt-handling function.
///
/// Assembly ISRs call into this, and it dispatches interrupts to
/// the appropriate consumers.
//  TODO: should this be #[cold]?
#[no_mangle] #[inline(never)]
pub extern "C" fn handle_interrupt(state: &InterruptContext) {
    let id = state.int_id;
    match id {
        // System timer
        0x20 => { /* TODO: make this work
                    TODO: should this IRQ get its own handler?
                  */ }
        // Keyboard
      , 0x21 => {
          // TODO: dispatch keypress event to subscribers (NYI)
            // TODO: should this interrupt get its own handler for perf?
            if let Some(input) = keyboard::read_char() {
                if input == '\r' {
                    println!("");
                } else {
                    print!("{}", input);
                }
            }
        }
        // Loonix syscall vector
      , 0x80 => { // TODO: currently, we do nothing here, do we want
                  // our syscalls on this vector as well?
        }
      , _ => panic!("Unknown interrupt: #{} Sorry!", id)
    }
    // send the PICs the end interrupt signal
    unsafe { pics::end_pic_interrupt(id as u8) };
}

#[no_mangle] #[inline(never)]
pub unsafe extern "C" fn keyboard_handler(state: &InterruptContext) {
    if let Some(input) = keyboard::read_char() {
        if input == '\r' {
            println!("");
        } else {
            print!("{}", input);
        }
    }
    // send the PICs the end interrupt signal
    pics::end_pic_interrupt(state.int_id as u8);
}

/// Handle a CPU exception with a given interrupt context.
//  TODO: should this be #[cold]?
#[no_mangle] #[inline(never)]
pub extern "C" fn handle_cpu_exception( state: &InterruptContext
                                      , err_code: usize) -> ! {
    let id = state.int_id;
    let ex_info = &EXCEPTIONS[id];
    let cr_state = control_regs::dump();
    let _ = write!( CONSOLE.lock()
                          .set_colors(Color::White, Color::Blue)
                        //   .clear()
                  , "CPU EXCEPTION {}: {}\n\
                     {} on vector {} with error code {:#x}\n\
                     Source: {}.\nThis is fine.\n\n"
                  , ex_info.mnemonic, ex_info.name, ex_info.irq_type
                  , id, err_code
                  , ex_info.source );

    // TODO: parse error codes
    let _ = match id {
        // TODO: page fault needs its own handler
        14 => write!( CONSOLE.lock()
                             .set_colors(Color::White, Color::Blue)
                    , "{}"
                    , PageFaultErrorCode::from_bits_truncate(err_code as u32)
                    )
       , _ => write!( CONSOLE.lock()
                             .set_colors(Color::White, Color::Blue)
                    , "Registers:\n{:?}\n    {}\n"
                    , state.registers
                    , cr_state
                    )
    };
    // TODO: stack dumps please

    loop { }
}

//isr! { interrupt 0x21
//     , keyboard_isr
//     , handler: "keyboard_handler" }

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
   IDT.lock()
      .add_handlers()
    //  .add_gate(0x21, keyboard_isr)
      .load();                 // Load the IDT pointer
   // print!("Testing interrupt handling...");
   // asm!("int $0" :: "N" (0x80));
   // println!("   [DONE]");
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
