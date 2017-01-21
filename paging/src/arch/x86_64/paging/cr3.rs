use super::table::{Table, PML4Level};
use memory::PhysicalPage;
use cpu::cr3::{read, write};

/// Returns the current Page Meta-Level 4 table
///
/// # Unsafe Because:
/// + Reading from control registers while not in kernel mode will cause
///   a general protection fault.
#[cfg(target_arch = "x86_64")]
#[inline]
// TODO: this needs to return a reference
pub unsafe fn current_pml4() -> Table<PML4Level> {
    use core::mem::transmute;
    //*read().as_mut_ptr::<Table<PML4Level>>()
    unimplemented!()
}

/// Sets the current Page Meta-Level 4 Table
///
/// # Unsafe Because:
/// + Control registers should generally not be modified during normal
///   operation.
#[cfg(target_arch = "x86_64")]
pub unsafe fn set_pml4(pml4: Table<PML4Level>) {
    write(pml4.frame().base_addr())
}

/// Returns the current Page Directory base frame.
///
/// # Unsafe Because:
/// + Reading from control registers while not in kernel mode will cause
///   a general protection fault.
pub unsafe fn current_pagetable_frame() -> PhysicalPage {
    PhysicalPage::containing_addr(read())
}

/// Returns the current Page Directory base frame.
///
/// # Unsafe Because:
/// + Reading from control registers while not in kernel mode will cause
///   a general protection fault.
#[inline]
pub unsafe fn set_pagetable_frame(frame: PhysicalPage) {
    write(frame.base_addr())
}
