use core::fmt;

bitflags! {
    pub flags Flags: usize {
        /// Protected Mode Enable
        ///
        /// If 1, system is in protected mode. Otherwise, the system is in real
        /// mode.
        const PE = 1 << 0
      , /// Monitor co-processor
        ///
        /// Controls interaction of `WAIT`/`FWAIT` instructions with `TS` flag
        /// in `%cr0`.
        const MP = 1 << 1
      , /// FPU Emulation
        ///
        /// If set, no x87 floating point unit present, if clear, x87 FPU
        /// present.
        const EM = 1 << 2
      , /// Task Switched
        ///
        /// Allows saving x87 task context upon a task switch only after x87
        /// instruction used.
        const TS = 1 << 3
      , /// Extension Type
        ///
        /// On a 386 CPU, indicated whether the math coprocessor was an 80287
        /// or an 80387.
        const ET = 1 << 4
      , /// Numeric Error
        ///
        /// Enable internal x87 floating point error reporting when set, else
        /// enables PC style x87 error detection.
        const NE = 1 << 5
      , /// Write Protect
        ///
        /// When set, the CPU can't write to read-only pages when privilege
        /// level is 0.
        const WP = 1 << 16
      , /// Alignment Mask
        ///
        /// Alignment check enabled if `AM` set, `AC` flag (in `%eflags`
        /// register) set, and privilege level is 3
        const AM = 1 << 18
      , /// Not Write-Through
        ///
        /// Globally enables/disable write-through caching
        const NW = 1 << 29
      , /// Cache Disable
        ///
        /// Globally enables/disable the memory cache
        const CD = 1 << 30
      , /// Paging
        ///
        /// If 1, enable paging and use the `%cr3` register, else disable
        /// paging.
        const PG = 1 << 31
    }
}

/// Set the write protect bit in `%cr0`.
pub fn enable_write_protect() {
    let mut flags: Flags = read();
    if !flags.contains(WP) {
        flags.insert(WP);
        unsafe { write(flags.bits) }
    }
}

/// Unset the write protect bit in `%cr0`.
pub fn disable_write_protect() {
    let mut flags: Flags = read();
    if flags.contains(WP) {
        flags.remove(WP);
        unsafe { write(flags.bits) }
    }
}

/// Set the paging bit in `%cr0`.
pub fn enable_paging() {
    let mut flags: Flags = read();
    if !flags.contains(WP) {
        flags.insert(WP);
        unsafe { write(flags.bits) }
    }
}

/// Unset the paging bit in `%cr0`.
pub fn disable_paging() {
    let mut flags: Flags = read();
    if flags.contains(PG) {
        flags.remove(PG);
        unsafe { write(flags.bits) }
    }
}

/// Read the current value from `%cr0`.
pub fn read() -> Flags {
    let result: usize;
    unsafe {
        asm!(   "mov $0, cr0"
            :   "=r"(result)
            ::: "intel" );
    }
    Flags { bits: result }
}

/// Write a value to `%cr0`.
///
/// # Unsafe Because:
///  - Control registers should generally not be modified during normal
///    operation.
pub unsafe fn write(flags: Flags) {
    asm!(  "mov cr0, $0"
        :: "r"(flags.bits)
        :: "intel");
}

impl fmt::LowerHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08x}", self.bits)
    }
}
