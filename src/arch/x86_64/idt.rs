use core::mem;
use spin::Mutex;

extern {
    /// Offset of the 64-bit GDT main code segment.
    /// This should be exported by ASM.
    static gdt64_offset: u16;
}

const IDT_ENTRIES: usize = 256;
type Handler = unsafe extern "C" fn() -> ();

/// An IDT entry is called a gate.
///
/// Based on code from the OS Dev Wiki
/// http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure
#[repr(C, packed)]
struct Gate { offset_lower: u16
            , selector: u16
            , zero: u8
            , type_attr: u8
            , offest_mid: u16
            , offset_upper: u32
            , reserved: u32
            }

impl Gate {
    /// Creates a new IDT gate marked as `absent`.
    ///
    /// This is basically just for filling the new IDT table
    /// with valid (but useless) gates upon init.
    const fn absent() -> Self {
        Gate { offset_lower: 0
             , selector: 0
             , zero: 0
             , type_attr: 0b0000_1110
             , offset_mid: 0
             , offset_upper: 0
             , reserved: 0
        }
    }

    fn new(handler: Handler) -> Gate {
        unsafe {
            // `mem::transmute()` is glorious black magic
            let (low, mid, high): (u16, u16, u32)
                = mem::transmute(handler)

            Gate { offset_lower: low
                 , selector: gdt64_offset
                 , zero: 0
                 , type_attr: 0b1000_1110
                 , offset_mid: mid
                 , offset_upper: high
                 , reserved: 0
                 }
        }
    }
}


struct Idt([Gate; IDT_ENTRIES]);

/// This is the format that `lidt` expects for the pointer to the IDT.
/// ...apparently.
#[repr(C, packed)]
struct IdtPtr { limit: u16
              , base: u64
              }

impl IdtPtr {
    /// Load the IDT at the given location.
    /// This just calls `lidt`.
    pub unsafe fn load(&self) {
        asm!(  "lidt ($0)"
            :: "{rax}"(self)
            :: "volatile" );
    }
}

/// Global Interrupt Descriptor Table instance
/// Our global IDT.
static IDT: Mutex<Idt>
    = Mutex::new(Idt([Gate::absent(); IDT_ENTRIES]));
