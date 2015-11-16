//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86 64-bit control registers

/// `cr0` contains flags that control the CPU's operations
pub unsafe fn cr0_read() -> u64 {
    let result: u64;
    asm!(   "mov $0, cr0"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr0_write(value: u64) {
    asm!(  "mov cr0, $0"
        :: "r"(value)
        :: "intel");
}

/// `cr2` contains the page fault linear address
pub unsafe fn cr2_read() -> u64 {
    let result: u64;
    asm!(   "mov $0, cr2"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr2_write(value: u64) {
    asm!(  "mov cr2, $0"
        :: "r"(value)
        :: "intel");
}
/// `cr3` contains the page table root pointer
pub unsafe fn cr3_read() -> u64 {
    let result: u64;
    asm!(   "mov $0, cr3"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr3_write(value: u64) {
    asm!(  "mov cr3, $0"
        :: "r"(value)
        :: "intel");
}

/// `cr4` contains flags that control operations in protected mode
pub unsafe fn cr4_read() -> u64 {
    let result: u64;
    asm!(   "mov $0, cr4"
        :   "=r"(result)
        ::: "intel" );
    result
}
pub unsafe fn cr4_write(value: u64) {
    asm!(  "mov cr4, $0"
        :: "r"(value)
        :: "intel");
}
