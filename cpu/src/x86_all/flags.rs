//! Flags present in the `%eflags`/`%rflags` register on x86 CPUs.
use super::PrivilegeLevel;

bitflags! {
    /// Contents of the `%eflags`/`%rflags` register.
    ///
    /// Note that on early x86 systems, this is a 16-bit register (`%flags`),
    /// but we only support the 32-bit `%eflags` and 64-bit `%rflags`, since
    /// SOS is a protected mode/long mode OS only.
    pub flags Flags: usize {
        /// Carry flag
        const CF = 1 << 0
      , /// Parity flag
        const PF = 1 << 2
      , /// Adjust flag
        const AF = 1 << 4
      , /// Zero flag
        const ZF = 1 << 6
      , /// Sign flag
        const SF = 1 << 7
      , /// Trap flag (single step)
        ///
        /// If 1, IT'S A TRAP!
        const TF = 1 << 8
      , /// Interrupt enable flag
        const IF = 1 << 9
      , /// Direction flag
        const DF = 1 << 10
      , /// Overflow flag
        const OF = 1 << 11

      , const IOPL_RING_0 = 0 << 12
      , const IOPL_RING_1 = 1 << 12
      , const IOPL_RING_2 = 2 << 12
      , const IOPL_RING_3 = 3 << 12
      , /// I/0 Privilege Level
        ///
        /// This flag is always one on the 8086 and 186.
        const IOPL = IOPL_RING_0.bits | IOPL_RING_1.bits |
                     IOPL_RING_2.bits | IOPL_RING_3.bits

      , /// Nested task flag
        ///
        /// Always 1 on 8086 and 186s.
        const NT = 1 << 14

      , /// Should always be 1
        const RESERVED = 1 << 15

      , /// Resume flag
        ///
        /// Present on 386 and later.
        const RF = 1 << 16
      , /// Virtual 8086 Mode flag
        ///
        /// Of course, this is only present on 386 and later.
        const VM = 1 << 17
      , /// Alignment Check
        ///
        /// Present on 486SX and later.
        const AC = 1 << 18
      , /// Virtual Interrupt flag
        ///
        /// Present on Pentium and later.
        const VIF = 1 << 19
      , /// Virtual Interrupt Pending
        ///
        /// Present on Pentium and later.
        const VIP = 1 << 20
      , /// Able to use `CPUID` instruction.
        ///
        /// Present on Pentium and later.
        const ID = 1 << 21
    }
}

impl Flags {
    pub fn iopl(&self) -> PrivilegeLevel {
        use core::mem::transmute;
        let bits = (*self & IOPL).bits >> 12;
        unsafe { transmute(bits as u16) }
    }
}

/// Read the current value from `$eflags`/`%rflags`.
pub fn read() -> Flags {
    let result: usize;
    unsafe {
        asm!(   "pushf
                 pop $0"
            :   "=r"(result)
            ::: "intel" );
    }
    Flags { bits: result }
}
