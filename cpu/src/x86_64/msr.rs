//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the Model-Specific Registers (MSRs).
use core::mem;

/// Extended Feature Enable Register (EFER) on IA-32
pub const IA32_EFER: u32 = 0xc0000080;

/// Write `value` to the specified `msr`
///
/// # Arguments
/// + `msr`: which MSR to write to
/// + `value`: the  bits to write
pub unsafe fn write(msr: u32, value: u64) {
    let (high, low): (u32, u32) = mem::transmute(value);
    asm!(   "wrmsr"
         :: "{ecx}" (msr), "{eax}" (low), "{edx}" (high)
         :  "memory"
         : "volatile" );
}

/// Read 64 bits from the specified `msr`
pub unsafe fn read(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!( "rdmsr"
        : "={eax}" (low), "={edx}" (high)
        : "{ecx}" (msr)
        : "memory"
        : "volatile" );
    mem::transmute((high,low))
}


/// Enable the NXE (No Execute) in the IA-32 EFER register.
///
/// This allows us to set the NXE bit on pages.
pub unsafe fn enable_nxe() {
    let nxe_bit = 1 << 11;
    let efer = read(IA32_EFER);
    write(IA32_EFER, efer | nxe_bit);
}
