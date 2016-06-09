//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! `x86` and `x86_64` control registers
use core::fmt;

bitflags! {
    /// Bitflags present in `$cr4`
    ///
    /// Documentation taken from
    /// [wikipedia](https://en.wikipedia.org/wiki/Control_register#CR4).
    pub flags Cr4Flags: usize {
        /// Virtual 8086 Mode Extensions
        ///
        /// If set, enables support for the virtual interrupt flag (VIF) in
        /// virtual-8086 mode.
        const CR4_VME = 1 << 0
      , /// Protected-mode Virtual Interrupts
        ///
        /// If set, enables support for the virtual interrupt flag (VIF) in
        /// protected mode.
        const CR4_PVI = 1 << 1
      , /// If 1, the `RTDSC` instruction can only be executed in Ring 0
        const CR4_TSD = 1 << 2
      , /// Debugging Extensions
        ///
        /// If set, enables debug register based breaks on I/O space access
        const CR4_DE = 1 << 3
      , /// Page Size Extension
        ///
        /// If unset, page size is 4 KiB, else page size is increased to 4 MiB
        /// (if PAE is enabled or the processor is in Long Mode this bit is
        /// ignored).
        const CR4_PSE = 1 << 4
      , /// Physical Address Extension
        ///
        /// If set, changes page table layout to translate 32-bit virtual
        /// addresses into extended 36-bit physical addresses.
        const CR4_PAE = 1 << 5
      , /// Machine Check Exception
        ///
        /// If set, Machine Check exceptions are enabled.
        const CR4_MCE = 1 << 6
      , /// Page Global Enabled
        ///
        /// If set, address translations (PDE or PTE records) may be shared
        /// between address spaces.
        const CR4_PGE = 1 << 7
      , /// Performance-Monitoring Counter enable
        ///
        /// If set, the `RDPMC` instruction can be executed at any privilege
        /// level, else `RDPMC` can only be used in ring 0.
        const CR4_PCE = 1 << 8
      , /// Operating system support for `FXSAVE` and `FXRSTOR` instructions
        ///
        /// If set, enables SSE instructions and fast FPU save and restore.
        const CR4_OSFXSR = 1 << 9
      , /// Operating System Support for Unmasked SIMD Floating-Point Exceptions
        ///
        /// If set, enables unmasked SSE exceptions.
        const CR4_OSXMMEXCPT = 1 << 10
      , /// Virtual Machine Extensions Enable
        const CR4_VMXE = 1 << 13
      , /// Safer Mode Extensions Enable
        const CR4_SMXE = 1 << 14
      , /// Enables the instructions RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE.
        const CR4_FSGSBASE = 1 << 16
      , /// PCID Enable
        ///
        /// If set, enables process-context identifiers (PCIDs).
        const CR4_PCIDE = 1 << 17
      , /// `XSAVE` and Processor Extended States Enable
        const CR4_OSXSAVE = 1 << 18
      , /// Supervisor Mode Execution Protection Enable
        ///
        /// If set, execution of code in a higher ring generates a fault
        const CR4_SMEP = 1 << 20
      , /// Supervisor Mode Access Protection Enable
        ///
        /// If set, access of data in a higher ring generates a faul
        const CR4_SMAP = 1 << 21
      , /// Protection Key Enable
        const CR4_PKE = 1 << 22
    }
}

impl fmt::LowerHex for Cr4Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08x}", self.bits)
    }
}

/// A struct bundling together a snapshot of the control registers state.
#[derive(Copy,Clone,Debug)]
pub struct CrState { /// `$cr0` contains flags that control the CPU's operations
                     pub cr0: usize
                   , /// `$cr2` contains the page fault linear address
                     pub cr2: usize
                   , /// `$cr3` contains the page table root pointer
                     pub cr3: usize
                   , /// `$cr4` contains flags that control operations in
                     ///  protected mode
                     pub cr4: Cr4Flags
                   }

impl fmt::Display for CrState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!( f, "CR0: {:#08x} CR2: {:#08x} CR3: {:#08x} CR4: {:#08x}"
                , self.cr0, self.cr2, self.cr3, self.cr4)
    }
}

/// Dump the current contents of the control registers to a `CrState`.
pub fn dump() -> CrState {
    let cr0_: usize; let cr2_: usize;
    let cr3_: usize; let cr4_: Cr4Flags;
    unsafe {
        asm!(  "mov $0, cr0
                mov $1, cr2
                mov $2, cr3
                mov $3, cr4"
            :   "=r"(cr0_)
              , "=r"(cr2_)
              , "=r"(cr3_)
              , "=r"(cr4_)
            ::: "intel"
              , "volatile");
    }
    CrState { cr0: cr0_, cr2: cr2_, cr3: cr3_, cr4: cr4_ }

}

/// Set the write protect bit in `cr0`.
pub fn set_write_protect() {
    let wp_bit = 1 << 16;
    unsafe { cr0_write(cr0_read() | wp_bit) };
}

/// Read the current value from `$cr0`.
pub fn cr0_read() -> usize {
    let result: usize;
    unsafe {
        asm!(   "mov $0, cr0"
            :   "=r"(result)
            ::: "intel" );
    }
    result
}

/// Write a value to `$cr0`.
///
/// # Unsafe Because:
///  - Control registers should generally not be modified during normal
///    operation.
pub unsafe fn cr0_write(value: usize) {
    asm!(  "mov cr0, $0"
        :: "r"(value)
        :: "intel");
}

/// Read the current value from `$cr2`.
pub fn cr2_read() -> usize {
    let result: usize;
    unsafe {
        asm!(   "mov $0, cr2"
            :   "=r"(result)
            ::: "intel" );
    }
    result
}

/// Write a value to `$cr2`.
///
/// # Unsafe Because:
///  - Control registers should generally not be modified during normal
///    operation.
pub unsafe fn cr2_write(value: usize) {
    asm!(  "mov cr2, $0"
        :: "r"(value)
        :: "intel");
}

/// Read the current value from `$cr3`.
pub fn cr3_read() -> usize {
    let result: usize;
    unsafe {
        asm!(   "mov $0, cr3"
            :   "=r"(result)
            ::: "intel" );
    }
    result
}

/// Write a value to `$cr3`.
///
/// # Unsafe Because:
///  - Control registers should generally not be modified during normal
///    operation.
pub unsafe fn cr3_write(value: usize) {
    asm!(  "mov cr3, $0"
        :: "r"(value)
        :: "intel");
}

/// Read the current value from `$cr4`.
pub fn cr4_read() -> Cr4Flags {
    let result: Cr4Flags;
    unsafe {
        asm!(   "mov $0, cr4"
            :   "=r"(result)
            ::: "intel" );
    }
    result
}

/// Write a value to `$cr4`.
///
/// # Unsafe Because:
///  - Control registers should generally not be modified during normal
///    operation.
pub unsafe fn cr4_write(value: usize) {
    asm!(  "mov cr4, $0"
        :: "r"(value)
        :: "intel");
}
