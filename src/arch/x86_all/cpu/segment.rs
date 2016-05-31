//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for using the x86_64 segmentation hardware.
//!
//! For more information, refer to the _Intel® 64 and IA-32 Architectures
//! Software Developer’s Manual_, Vol. 3A, section 3.2, "Using Segments".
//! Some of the documentation present in this module was taken from the Intel
//! manual.
use core::fmt;

/// A segment selector is a 16-bit identifier for a segment.
///
/// It does not point directly to the segment, but instead points to the
/// segment descriptor that defines the segment.
///
/// A segment selector contains the following items:
///     - *Requested Privilege Level (RPL)*: bits 0 and 1.
///       Specifies the privelege level of the selector.
///     - *Table Indicator*: bit 2. Specifies which descriptor table to use.
///     - *Index*: bits 3 through 15. Selects one of 8192 descriptors in the
///       GDT or LDT. The processor multiplies the index value by 8 (the number
///       of bytes in a segment descriptor) and adds the result to the base
///       address of the GDT or LDT (from the GDTR or LDTR register,
///       respectively).
bitflags! {
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

    /// Extracts the index from a segment selector
    #[inline] pub fn index(&self) -> u16 {
        self.bits >> 3
    }

    /// Load this selector into the stack segment register.
    pub unsafe fn load_ss(&self) {
        asm!(  "mov ss, $0"
            :: "r"(self.bits)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the data segment register.
    pub unsafe fn load_ds(&self) {
        asm!(  "mov ds, $0"
            :: "r"(self.bits)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the es segment register.
    pub unsafe fn load_es(&self) {
        asm!(  "mov es, $0"
            :: "r"(self.bits)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the fs segment register.
    pub unsafe fn load_fs(&self) {
        asm!(  "mov fs, $0"
            :: "r"(self.bits)
            :  "memory"
            :  "intel");
    }

    /// Load this selector into the gs segment register.
    pub unsafe fn load_gs(&self) {
        asm!(  "mov gs, $0"
            :: "r"(self.bits)
            :  "memory"
            :  "intel");
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this could be much less ugly.
        let ring = if self.contains(RPL_RING_0) { "0" }
                   else if self.contains(RPL_RING_1) { "1" }
                   else if self.contains(RPL_RING_2) { "2" }
                   else if self.contains(RPL_RING_3) { "3" }
                   else { unreachable!() };
        let table = if self.contains(TI_GDT) { "GDT" }
                    else { "LDT" };
        write!(f, "Index {} in {} at Ring {}", self.index(), table, ring)
    }
}
