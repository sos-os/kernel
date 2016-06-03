use arch::cpu::Registers;
use arch::cpu::dtable::DTable;

use io::keyboard;

use spin::Mutex;

pub mod idt;
pub mod pics;

use self::idt::{Idt, Gate};

/// State stored when handling an interrupt.
#[repr(C, packed)]
pub struct InterruptContext { /// callee-saved registers
                              pub registers: Registers
                            , /// interrupt ID number
                              pub int_id:  u32
                            , _pad_1: u32
                            , /// error number
                              pub err_no:  u32
                            , _pad_2: u32
                            }

impl InterruptContext {
    /// Fetches the corresponding CPU exception for this interrupt, if this
    /// interrupt is a CPU exception.
    #[inline]
    pub fn exception(&self) -> &ExceptionInfo {
        &EXCEPTIONS[self.int_id as usize]
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


//==--------------------------------------------------------------------------==
// Top-level interrupt handling
/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt> = Mutex::new(Idt::new());

/// Kernel interrupt-handling function.
///
/// Assembly interrupt handlers call into this, and it dispatches interrupts to
/// the appropriate consumers.
#[no_mangle]
pub unsafe extern "C" fn handle_interrupt(state: &InterruptContext) {
   let id = state.int_id;
   match id {
       // interrupts 0 - 16 are CPU exceptions
       0x00...0x0f => Idt::handle_cpu_exception(state)
       // System timer
     , 0x20 => { /* TODO: make this work */ }
       // Keyboard
     , 0x21 => {
         // TODO: dispatch keypress event to subscribers (NYI)
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
   pics::end_pic_interrupt(id as u8);
}

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
   IDT.lock()
      .add_handlers()
      .load();                 // Load the IDT pointer
   // print!("Testing interrupt handling...");
   // asm!("int $0" :: "N" (0x80));
   // println!("   [DONE]");
   Idt::enable_interrupts(); // enable interrupts

}
