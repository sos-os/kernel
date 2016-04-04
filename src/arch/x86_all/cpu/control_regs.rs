//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86 64-bit control registers
use core::fmt;

#[derive(Copy,Clone,Debug)]
pub struct CrState { pub cr0: usize, pub cr2: usize
                   , pub cr3: usize, pub cr4: usize
                   }

impl fmt::Display for CrState {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!( f, "CR0: {:#08x} CR2: {:#08x} CR3: {:#08x} CR4: {:#08x}"
                , self.cr0, self.cr2, self.cr3, self.cr4)
     }
}

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
    CrState { cr0: cr0_, cr2: cr2_, cr3: cr3_, cr4: cr4_ }

}

/// Set the write protect bit in `cr0`.
pub fn set_write_protect() {
    let wp_bit = 1 << 16;
    unsafe { cr0_write(cr0_read() | wp_bit) };
}

/// `cr0` contains flags that control the CPU's operations
pub unsafe fn cr0_read() -> usize {
    let result: usize;
    asm!(   "mov $0, cr0"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr0_write(value: usize) {
    asm!(  "mov cr0, $0"
        :: "r"(value)
        :: "intel");
}

/// `cr2` contains the page fault linear address
pub unsafe fn cr2_read() -> usize {
    let result: usize;
    asm!(   "mov $0, cr2"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr2_write(value: usize) {
    asm!(  "mov cr2, $0"
        :: "r"(value)
        :: "intel");
}
/// `cr3` contains the page table root pointer
pub unsafe fn cr3_read() -> usize {
    let result: usize;
    asm!(   "mov $0, cr3"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr3_write(value: usize) {
    asm!(  "mov cr3, $0"
        :: "r"(value)
        :: "intel");
}

/// `cr4` contains flags that control operations in protected mode
pub unsafe fn cr4_read() -> usize {
    let result: usize;
    asm!(   "mov $0, cr4"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr4_write(value: usize) {
    asm!(  "mov cr4, $0"
        :: "r"(value)
        :: "intel");
}
