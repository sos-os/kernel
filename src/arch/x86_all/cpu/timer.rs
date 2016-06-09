pub mod timestamp {
    use core::mem;


    /// Read the current value of the timestamp counter.
    ///
    /// # Unsafe Because:
    /// + This will cause a General Protection Fault if the TSD flag in register
    ///   `%cr4` is set and the CPL is greater than 0.
    pub unsafe fn rtdsc() -> u64 {
        let (high, low): (u32, u32);
        asm!( "rdtsc"
            : "={eax}" (low), "={edx}" (high));
        mem::transmute((high, low))
    }

    /// Read the current timestamp, after other instructions have been executed.
    ///
    /// # Unsafe Because:
    /// + This will cause a General Protection Fault if the TSD flag in register
    ///   `%cr4` is set and the CPL is greater than 0.
    pub unsafe fn rtdscp() -> u64 {
        let (high, low): (u32, u32);
        asm!( "rdtscp"
            : "={eax}" (low), "={edx}" (high)
            ::: "volatile");
        mem::transmute((high, low))
    }

    /// Returns true if timestamps are currently available.
    #[inline]
    pub fn is_available() -> Result<(), &'static str> {
        use arch::cpu::control_regs::cr4;
        use arch::cpu::PrivilegeLevel;

        if cr4::is_timestamp_disabled() {
            Err("Timestamp Disable bit in %cr4 is set")
        } else if PrivilegeLevel::current_iopl() != PrivilegeLevel::KernelMode {
            Err("Insufficient privilege level to get timestamp, must be Ring 0")
        } else { Ok(()) }
    }

    /// Returns the current timestamp, or an error
    #[inline]
    pub fn get_timestamp() -> Result<u64, &'static str> {
        is_available().map(|_| unsafe { rtdsc() })
    }

    /// Returns the current timestamp or an error, after other instructions have
    /// been executed.
    #[inline]
    pub fn wait_get_timestamp() -> Result<u64, &'static str> {
        is_available().map(|_| unsafe { rtdscp() })
    }
}
