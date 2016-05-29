//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for the `x86` and `x86_64` Interrupt Descriptor Table.

use core::{fmt, mem};
use core::fmt::Write;

use memory::PAddr;

use super::super::{dtable, control_regs};
use super::super::dtable::DTable;
use io::term::CONSOLE;

use vga::Color;


pub type Handler = unsafe extern "C" fn() -> !;
pub const IDT_ENTRIES: usize = 256;

/// x86 interrupt gate types.
///
/// Bit-and this with the attribute half-byte to produce the
/// `type_attr` field for a `Gate`
#[repr(u8)]
#[derive(Copy,Clone,Debug)]
pub enum GateType { Absent    = 0b0000_0000
                  , Interrupt = 0b1000_1110
                  , Call      = 0b1000_1100
                  , Trap      = 0b1000_1111
                  }

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self { GateType::Absent    => write!(f, "Absent")
                    , GateType::Interrupt => write!(f, "Interrupt")
                    , GateType::Call      => write!(f, "Call")
                    , GateType::Trap      => write!(f, "Trap")
                    }
    }
}

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

pub trait Gate {
    fn from_handler(handler: Handler) -> Self;
    unsafe fn from_raw(handler: *const u8) -> Self;
}

// /// This is the format that `lidt` expects for the pointer to the IDT.
// /// ...apparently.
// #[repr(C, packed)]
// pub struct IdtPtr<I>
// where I: Idt { pub limit: u16
//              , pub base: *const I
//              }
//
// pub trait IdtPtrOps {
//     unsafe fn load(&self);
// }

pub trait InterruptContext {
    type Registers: fmt::Debug;

    fn err_no(&self) -> u32;
    fn int_id(&self) -> u32;
    fn registers(&self) -> Self::Registers;

    #[inline]
    fn exception(&self) -> &ExceptionInfo {
        &EXCEPTIONS[self.int_id() as usize]
    }


}

pub trait Idt: Sized {
    type Ctx: InterruptContext;
    type GateSize: Gate;

    /// Get the IDT pointer struct to pass to `lidt`
    fn get_ptr(&self) -> dtable::Pointer {
            dtable::Pointer {
                limit: (mem::size_of::<Self::GateSize>() * IDT_ENTRIES) as u16
              , base: PAddr::from_ptr(unsafe { mem::transmute(self) })
            }
        }

    /// Enable interrupts
    unsafe fn enable_interrupts() { asm!("sti") }
    /// Disable interrupts
    unsafe fn disable_interrupts() { asm!("cli") }

    /// Add a new interrupt gate pointing to the given handler
    fn add_gate(&mut self, idx: usize, handler: Handler);

    unsafe fn handle_cpu_exception(state: &Self::Ctx) -> ! {
        let ex_info = state.exception();
        let cr_state = control_regs::dump();
        let _ = write!( CONSOLE.lock()
                              .set_colors(Color::White, Color::Blue)
                            //   .clear()
                      , "CPU EXCEPTION {}: {}\n\
                         {} on vector {} with error code {:#x}\n\
                         Source: {}.\nThis is fine.\n\n"
                      , ex_info.mnemonic, ex_info.name
                      , ex_info.irq_type, state.int_id(), state.err_no()
                      , ex_info.source );

        // TODO: parse error codes
        let _ = match state.int_id() {
            14 => unimplemented!() //TODO: special handling for page faults
           , _ => write!( CONSOLE.lock()
                                 .set_colors(Color::White, Color::Blue)
                        , "Registers:\n{:?}\n    {}\n"
                        , state.registers()
                        , cr_state
                        )
        };

        loop { }
    }

    //unsafe extern "C" fn handle_interrupt(state: &Self::Ctx);
}

impl<I: Idt> DTable for I {
    #[inline] unsafe fn load(&self) {
        asm!(  "lidt ($0)"
            :: "r"(&self.get_ptr())
            :  "memory" );
        println!("{:<38}{:>40}", " . . Loading IDT", "[ OKAY ]");
    }
}
