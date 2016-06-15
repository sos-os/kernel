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

/// `%cr0` contains flags that modify basic processor operation.
pub mod cr0;

/// `%cr4` contains flags that control protected mode execution.
pub mod cr4;


/// A struct bundling together a snapshot of the control registers state.
#[derive(Copy,Clone,Debug)]
pub struct CrState { /// `$cr0` contains flags that control the CPU's operations
                     pub cr0: cr0::Flags
                   , /// `$cr2` contains the page fault linear address
                     pub cr2: usize
                   , /// `$cr3` contains the page table root pointer
                     pub cr3: usize
                   , /// `$cr4` contains flags that control operations in
                     ///  protected mode
                     pub cr4: cr4::Flags
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
    let cr3_: usize; let cr4_: usize;
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
    CrState { cr0: cr0::Flags::from_bits_truncate(cr0_)
            , cr2: cr2_, cr3: cr3_
            , cr4: cr4::Flags::from_bits_truncate(cr4_)
            }

}

pub mod cr2 {

    /// Read the current value from `$cr2`.
    ///
    /// # Unsafe Because:
    /// + Reading from control registers while not in kernel mode will cause
    ///   a general protection fault.
    pub unsafe fn read() -> usize {
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
    /// + Control registers should generally not be modified during normal
    ///   operation.
    pub unsafe fn write(value: usize) {
        asm!(  "mov cr2, $0"
            :: "r"(value)
            :: "intel");
    }
}

pub mod cr3 {
    use memory::paging::PhysicalPage;

    /// Read the current value from `$cr3`.
    ///
    /// # Unsafe Because:
    /// + Reading from control registers while not in kernel mode will cause
    ///   a general protection fault.
    pub fn read() -> usize {
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
    /// + Control registers should generally not be modified during normal
    ///   operation.
    pub unsafe fn write(value: usize) {
        asm!(  "mov cr3, $0"
            :: "r"(value)
            :: "intel");
    }

    /// Returns the current Page Directory base frame.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn pd_base_frame() -> PhysicalPage {
        use memory::PAddr;
        PhysicalPage::containing_addr(PAddr::from(read() as u64))
    }

    /// Returns the current Page Directory base frame.
    #[cfg(target_arch = "x86")]
    pub unsafe fn pd_base_frame() -> PhysicalPage {
        use memory::PAddr;
        PhysicalPage::containing_addr(PAddr::from(read() as u32))
    }
}
