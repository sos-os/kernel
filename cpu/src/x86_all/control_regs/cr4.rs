use core::fmt;

/// Read the current value from `$cr4`.
///
/// # Unsafe Because:
/// + Reading from control registers while not in kernel mode will cause
///   a general protection fault.
pub unsafe fn read() -> Flags {
    let result: usize;
    asm!(   "mov $0, cr4"
        :   "=r"(result)
        ::: "intel" );
    Flags { bits: result }
}

/// Write a value to `$cr4`.
///
/// # Unsafe Because:
/// + Control registers should generally not be modified during normal
///   operation.
pub unsafe fn write(flags: Flags) {
    asm!(  "mov cr4, $0"
        :: "r"(flags.bits)
        :: "intel");
}


bitflags! {
    /// Bitflags present in `$cr4`
    ///
    /// Documentation taken from
    /// [wikipedia](https://en.wikipedia.org/wiki/Control_register#CR4).
    pub flags Flags: usize {
        /// Virtual 8086 Mode Extensions
        ///
        /// If set, enables support for the virtual interrupt flag (VIF) in
        /// virtual-8086 mode.
        const VME = 1 << 0
      , /// Protected-mode Virtual Interrupts
        ///
        /// If set, enables support for the virtual interrupt flag (VIF) in
        /// protected mode.
        const PVI = 1 << 1
      , /// Time Stamp Disable
        ///
        /// If 1, the `RTDSC` instruction can only be executed in Ring 0
        const TSD = 1 << 2
      , /// Debugging Extensions
        ///
        /// If set, enables debug register based breaks on I/O space access
        const DE = 1 << 3
      , /// Page Size Extension
        ///
        /// If unset, page size is 4 KiB, else page size is increased to 4 MiB
        /// (if PAE is enabled or the processor is in Long Mode this bit is
        /// ignored).
        const PSE = 1 << 4
      , /// Physical Address Extension
        ///
        /// If set, changes page table layout to translate 32-bit virtual
        /// addresses into extended 36-bit physical addresses.
        const PAE = 1 << 5
      , /// Machine Check Exception
        ///
        /// If set, Machine Check exceptions are enabled.
        const MCE = 1 << 6
      , /// Page Global Enabled
        ///
        /// If set, address translations (PDE or PTE records) may be shared
        /// between address spaces.
        const PGE = 1 << 7
      , /// Performance-Monitoring Counter enable
        ///
        /// If set, the `RDPMC` instruction can be executed at any privilege
        /// level, else `RDPMC` can only be used in ring 0.
        const PCE = 1 << 8
      , /// Operating system support for `FXSAVE` and `FXRSTOR` instructions
        ///
        /// If set, enables SSE instructions and fast FPU save and restore.
        const OSFXSR = 1 << 9
      , /// Operating System Support for Unmasked SIMD Floating-Point Exceptions
        ///
        /// If set, enables unmasked SSE exceptions.
        const OSXMMEXCPT = 1 << 10
      , /// Virtual Machine Extensions Enable
        const VMXE = 1 << 13
      , /// Safer Mode Extensions Enable
        const SMXE = 1 << 14
      , /// Enables the instructions RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE.
        const FSGSBASE = 1 << 16
      , /// PCID Enable
        ///
        /// If set, enables process-context identifiers (PCIDs).
        const PCIDE = 1 << 17
      , /// `XSAVE` and Processor Extended States Enable
        const OSXSAVE = 1 << 18
      , /// Supervisor Mode Execution Protection Enable
        ///
        /// If set, execution of code in a higher ring generates a fault
        const SMEP = 1 << 20
      , /// Supervisor Mode Access Protection Enable
        ///
        /// If set, access of data in a higher ring generates a faul
        const SMAP = 1 << 21
      , /// Protection Key Enable
        const PKE = 1 << 22
    }
}

impl fmt::LowerHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08x}", self.bits)
    }
}

cpu_flag! {
    doc="If disabled, the `RTDSC` instruction can only be executed in Ring 0.",
    TSD, is_timestamp_disabled, disable_timestamp
}
