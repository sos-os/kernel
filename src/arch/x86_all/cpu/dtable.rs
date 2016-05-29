
/// A pointer to a descriptor table (IDT or GDT)
#[repr(C, packed)]
pub struct Pointer { pub limit: u16
                   , pub base: usize
                   }

/// A descriptor table (IDT or GDT)
pub trait DTable {
    unsafe fn load(&self);
}
