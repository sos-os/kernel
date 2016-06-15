use memory::VAddr;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
///
/// # Unsafe Because
/// + Causes a general protection fault if not executed in kernel mode.
pub unsafe fn flush_addr(addr: VAddr) {
    asm!("invlpg ($0)" :: "r" (addr) : "memory");
}
