//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the x86_64 CPU.
//!
//! Currently this module contains a quick implementation of CPU port
//! input and output, and little else.
//!

#[path = "../x86_all/cpu.rs"] mod cpu_all;
#[path = "../x86_all/pics.rs"] pub mod pics;

pub use self::cpu_all::*;

use core::mem;

pub mod control_regs {
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
}


/// Registers pushed to the stack when handling an interrupt or context switch.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Registers { pub rsi: u64
                     , pub rdi: u64
                     , pub r11: u64
                     , pub r10: u64
                     , pub r9:  u64
                     , pub r8:  u64
                     , pub rdx: u64
                     , pub rcx: u64
                     , pub rax: u64
                     }

 impl Registers {
     // Transform this struct into an array of u64s
     // (if you would ever want to do this)
     pub unsafe fn to_array(&self) -> [u64; 9] {
        //  [ self.rsi, self.rdi, self.r11
        //  , self.r10, self.r9, self.r8
        //  , self.rdx, self.rcx, self.rax
        //  ]
        // using transmute is probably faster and we're already unsafe...
        mem::transmute(*self)
     }

     /// Create a new empty set of Registers
     pub const fn empty() -> Self {
         Registers { rsi: 0, rdi: 0, r11: 0
                   , r10: 0, r9:  0, r8:  0
                   , rdx: 0, rcx: 0, rax: 0
                   }
     }
 }

/// Thread execution context
#[repr(C, packed)]
pub struct Context { /// Value of the stack pointer (`rsp`) register
                     pub rsp: *mut u8
                   , /// Value of the caller-saved registers
                     pub registers: Registers
                   , /// Value of the instruction pointer (`rip`) register
                     pub rip: *mut u8
                 //, pub stack: [u8] // TODO: should be box
                   }

impl Context {
    pub fn empty() -> Self {
        unsafe {
            Context { rsp: mem::transmute(0u64)
                    , registers: Registers::empty()
                    , rip: mem::transmute(0u64)
                  //, stack: [0u8; 8]
                    }
        }
    }
}
