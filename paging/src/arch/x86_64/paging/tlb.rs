use ::VAddr;
use super::{Page, VirtualPage};

/// Invalidate the TLB completely by reloading the CR3 register.
///
/// # Unsafe Because
/// + Causes a general protection fault if not executed in kernel mode.
pub unsafe fn flush_all() {
    use arch::cpu::control_regs::cr3;
    cr3::write(cr3::read());
}

/// Something which may be flushed from the TLB
pub trait Flush {
    /// Invalidate this object in the TLB using the `invlpg` instruction.
    ///
    /// # Unsafe Because
    /// + Causes a general protection fault if not executed in kernel mode.
    unsafe fn invlpg(&self);

    /// Invalidate this object in the TLB instruction if we are in kernel mode.
    ///
    /// # Returns
    /// + True if the object was flushed
    /// + False if it was not flushed.
    #[inline]
    fn flush(&self) -> bool {
        use arch::cpu::PrivilegeLevel;

        if PrivilegeLevel::current_iopl() != PrivilegeLevel::KernelMode {
            false // can't flush, we are not in kernel mode
        } else {
            // this is safe since we know we are in kernel mode
            unsafe { self.invlpg() };
            true
        }
    }
}

impl Flush for VAddr {
    #[inline]
    unsafe fn invlpg(&self) {
         asm!( "invlpg [$0]"
             :
             : "r" (*self)
             : "memory"
             : "intel", "volatile" );
    }
}

impl Flush for VirtualPage {
    #[inline]
    unsafe fn invlpg(&self) {
        self.base().invlpg()
    }
}
