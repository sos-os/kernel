//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for using the `x86` and `x86_64` segmentation hardware.
//!
//! For more information, refer to the _Intel® 64 and IA-32 Architectures
//! Software Developer’s Manual_, Vol. 3A, section 3.2, "Using Segments".
//! Some of the documentation present in this module was taken from the Intel
//! manual.
use core::{fmt, mem};
use super::PrivilegeLevel;

pub mod gdt;

bitflags! {
    /// A segment selector is a 16-bit identifier for a segment.
    ///
    /// It does not point directly to the segment, but instead points to the
    /// segment descriptor that defines the segment.
    ///
    /// A segment selector contains the following items:
    ///
    /// + *Requested Privilege Level (RPL)*: bits 0 and 1.
    ///    Specifies the privelege level of the selector.
    /// + *Table Indicator*: bit 2. Specifies which descriptor table to use.
    /// + *Index*: bits 3 through 15. Selects one of 8192 descriptors in the
    ///    GDT or LDT. The processor multiplies the index value by 8 (the number
    ///    of bytes in a segment descriptor) and adds the result to the base
    ///    address of the GDT or LDT (from the `%gdtr` or `%ldtr` register,
    ///    respectively).
    pub flags Selector: u16 { const RPL_RING_0 = 0b00
                            , const RPL_RING_1 = 0b01
                            , const RPL_RING_2 = 0b10
                            , const RPL_RING_3 = 0b11

                            , /// Requested Prrivelege Level (RPL)
                              const RPL = RPL_RING_0.bits
                               | RPL_RING_1.bits
                               | RPL_RING_2.bits
                               | RPL_RING_3.bits

                            , /// If the Table Indicator (TI) is 0, use the GDT
                              const TI_GDT = 0 << 3

                            , /// If the TI is 1, use the LDT
                              const TI_LDT = 1 << 3
                            }
}

impl Selector {
    /// Create a new `Selector`
    ///
    /// # Arguments:
    ///   - `index`: the index in the GDT or LDT
    pub const fn new(index: u16) -> Self {
        Selector { bits: index << 3 }
    }

    /// Create a new `Selector` from raw bits
    pub const fn from_raw(bits: u16) -> Self {
        Selector { bits: bits }
    }

    /// Returns the current value of the code segment register.
    pub fn from_cs() -> Self {
        let cs: u16;
        unsafe {
            asm!( "mov $0, cs"
                : "=r"(cs)
                ::: "intel" )
        };
        Selector::from_bits_truncate(cs)
    }

    /// Extracts the index from a segment selector
    #[inline] pub fn index(&self) -> u16 {
        self.bits >> 3
    }

    /// Sets this segment selector to be a GDT segment.
    ///
    /// If the segment is already a GDT segment, this will quietly do nothing.
    #[inline] pub fn set_global(&mut self) -> &mut Self {
        self.remove(TI_LDT);
        self
    }

    /// Sets this segment selector to be an LDT segment.
    ///
    /// If the segment is already an LDT segment, this will quietly do nothing.
    #[inline] pub fn set_local(&mut self) -> &mut Self {
        self.insert(TI_GDT);
        self
    }

    /// Sets the Requested Priveliege Level (RPL)
    ///
    /// The RPL must be in the range between 0 and 3.
    #[inline] pub fn set_rpl(&mut self, rpl: PrivilegeLevel) -> &mut Self {
        self.bits &= rpl as u16;
        self
    }

    /// Checks the segment's privelige.
    #[inline] pub fn get_rpl(&self) -> PrivilegeLevel {
        unsafe { mem::transmute(*self & RPL) }
    }


    /// Load this selector into the stack segment register (`ss`).
    pub unsafe fn load_ss(&self) {
        asm!(  "mov ss, $0"
            :: "r"(*self)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the data segment register (`ds`).
    pub unsafe fn load_ds(&self) {
        asm!(  "mov ds, $0"
            :: "r"(*self)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the `es` segment register.
    pub unsafe fn load_es(&self) {
        asm!(  "mov es, $0"
            :: "r"(*self)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the `fs` segment register.
    pub unsafe fn load_fs(&self) {
        asm!(  "mov fs, $0"
            :: "r"(*self)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the `gs` segment register.
    pub unsafe fn load_gs(&self) {
        asm!(  "mov gs, $0"
            :: "r"(*self)
            :  "memory"
            :  "intel");
    }


    /// Load this selector into the code segment register.
    ///
    /// N.B. that as we cannot `mov` directly to `cs`, we have to do this
    /// differently. We push the selector and return value onto the stack,
    /// and use `lret` to reload `cs`.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn load_cs(&self) {
        asm!(  "push $0
                lea rax, [rip + 1]
                push rax
                iret
                1:"
            :: "r"(self.bits as u64)
            :  "rax", "memory"
            :  "intel");
    }

}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this could be much less ugly.
        let ring = if self.contains(RPL_RING_3) { "3" }
                   else if self.contains(RPL_RING_2) { "2" }
                   else if self.contains(RPL_RING_1) { "1" }
                   else if self.contains(RPL_RING_0) { "0" }
                   else { unreachable!() };
        let table = if self.contains(TI_GDT) { "GDT" }
                    else { "LDT" };
        write!(f, "{}[{}], Ring {}", table, self.index(), ring)
    }
}

/// A segment descriptor is an entry in an IDT or GDT.
///
/// A segment descriptor is a data structure in a GDT or LDT that provides the
/// processor with the size and location of a segment, as well as access control
/// and status information. Segment descriptors are typically created by
/// compilers, linkers, loaders, or the operating system or executive, but not
/// application programs.
///
#[repr(C, packed)]
pub struct Descriptor { /// The last 8 bits of the base address
                        pub base_high: u8
                      , /// The next 16 bits are bitflags
                        pub flags: Flags
                      , /// The middle 8 bits of the base address
                        pub base_mid: u8
                      , /// The first 16 bits of the base address
                        pub base_low: u16
                      , /// the segment limit
                        pub limit: u16
                      }

impl Descriptor {

    /// Constructs a new null `Descriptor`
    pub const fn null() -> Self {
        Descriptor { base_high: 0
                   , flags: Flags::null()
                   , base_mid: 0
                   , base_low: 0
                   , limit: 0
                   }
    }

    /// Constructs a new `Descriptor` from a `limit` and a `base` address
    pub fn new(base: u32, limit: u32) -> Self {
        let (hi, mid, lo): (u8, u8, u16) = unsafe { mem::transmute(base) };
        let (limit_lo, limit_hi): (u16, u16) = unsafe { mem::transmute(limit) };
        // I hope this is right...
        let flags = (limit_hi & 0b1111) << 8;

        Descriptor { base_high: hi
                   , flags: Flags::from_bits_truncate(flags)
                   , base_mid: mid
                   , base_low: lo
                   , limit: limit_lo
                   }
    }

    /// Extract the limit part from the flags and limit fields.
    #[inline]
    pub fn get_limit(&self) -> u32 {
        // TODO: i hope this is right...
        self.flags.get_limit_part() & self.limit as u32
    }


}


bitflags! {
    pub flags Flags: u16 {
        /// 1 if this is a code or data segment that has been accessed
        const CODE_DATA_ACC = 1 << 0
      , const SEGMENT_TYPE  = 0b0000_0000_0000_1111
      , const DESCR_TYPE    = 1 << 4
      , const DPL           = 0b0000_0000_0110_0000
      , const PRESENT       = 1 << 7
      , const LIMIT         = 0b0000_1111_0000_0000
      , const AVAILABLE     = 1 << 12
      , const LENGTH        = 1 << 13
      , const DEFAULT_SIZE  = 1 << 14
      , const GRANULARITY   = 1 << 15
      , /// If this is a code or data segment and the accessed bit is set,
        /// it has been accessed.
        const ACCESSED      = DESCR_TYPE.bits & CODE_DATA_ACC.bits
    }
}

impl Flags {

    const fn null() -> Self {
        Flags { bits: 0 }
    }

    /// Get the Descriptor Privilege Level (DPL) from the flags
    #[inline] pub fn get_dpl(&self) -> PrivilegeLevel {
        unsafe { mem::transmute((*self & DPL).bits >> 5) }
    }

    /// Returns true if this segment is a system segment.
    ///
    /// Returns false if it is a code or data segment.
    #[inline] pub fn is_system(&self) -> bool {
        !self.contains(DESCR_TYPE)
    }

    /// Returns false if this segment is present
    #[inline] pub fn is_present(&self) -> bool {
        !self.contains(PRESENT)
    }

    /// Returns false if this segment is available to system software
    #[inline] pub fn is_available(&self) -> bool {
        self.contains(AVAILABLE)
    }

    /// Returns true if this is a code or data segment that has been accessed.
    ///
    /// Returns false if it has not been accessed OR if it is a system segment.
    #[inline] pub fn is_accessed(&self) -> bool {
        self.contains(ACCESSED)
    }

    /// Returns the system type indicator, if this is a system segment.
    ///
    /// # Returns
    /// + `Some(SysType)` if this is a system segment
    /// + `None` if this is a code or data segment
    pub fn get_system_type(&self) -> Option<SysType> {
        if self.is_system() {
            Some(unsafe { mem::transmute((*self & SEGMENT_TYPE).bits) })
        } else {
            None
        }
    }

    /// Returns the code type indicator.
    ///
    /// # Returns
    /// + `Some(CodeType)` if this is not a system segment
    /// + `None` if this is a system segment
    pub fn get_code_type(&self) -> Option<CodeFlags> {
        if self.is_system() {
            None
        } else {
            Some(CodeFlags::from_bits_truncate(self.bits))
        }
    }

    /// Returns the data type indicator.
    ///
    /// # Returns
    /// + `Some(DataType)` if this is not a system segment
    /// + `None` if this is a system segment
    pub fn get_data_type(&self) -> Option<DataFlags> {
        if self.is_system() {
            None
        } else {
            Some(DataFlags::from_bits_truncate(self.bits))
        }
    }

    #[inline]
    fn get_limit_part(&self) -> u32 {
        ((*self & LIMIT).bits as u32) << 8
    }

}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Type { System(SysType)
              , Code(CodeFlags)
              , Data(DataFlags)
              }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u16)]
pub enum SysType { Ldt           = 0b0010
                 , TssAvailable  = 0b1001
                 , TssBusy       = 0b1011
                 , CallGate      = 0b1100
                 , InterruptGate = 0b1110
                 , TrapGate      = 0b1111
                 }

bitflags! {
    pub flags DataFlags: u16 {
        const DATA_ACCESSED = 0b0001
      , const WRITE         = 0b0010
      , const EXPAND_DOWN   = 0b0100
    }
}

impl DataFlags {
    /// Returns true if the data segment is read-only
    #[inline] pub fn is_read_only(&self) -> bool {
        !self.contains(WRITE)
    }

    /// Returns true if the data segment has been accessed
    #[inline] pub fn is_accessed(&self) -> bool {
        self.contains(DATA_ACCESSED)
    }

    /// Returns true if the data segment expands down
    #[inline] pub fn is_expand_down(&self) -> bool {
        self.contains(EXPAND_DOWN)
    }
}

bitflags! {
    pub flags CodeFlags: u16 {
        const CODE_ACCESSED = 0b0001
      , const READ          = 0b0010
      , const EXECUTE       = 0b1000
      , const CONFORMING    = 0b0100
      , const EXEC_ONLY     = EXECUTE.bits & !READ.bits
    }
}

impl CodeFlags {
    /// Returns true if the code segment is execute-only (not readable)
    #[inline] pub fn is_exec_only(&self) -> bool {
        self.contains(EXEC_ONLY)
    }

    /// Returns true if the code segment is readable
    #[inline] pub fn is_readable(&self) -> bool {
        self.contains(READ)
    }

    /// Returns true if the code segment has been accessed.
    #[inline] pub fn is_accessed(&self) -> bool {
        self.contains(CODE_ACCESSED)
    }

    /// Returns true if the code segment is conforming.
    #[inline] pub fn is_conforming(&self) -> bool {
        self.contains(CONFORMING)
    }
}
