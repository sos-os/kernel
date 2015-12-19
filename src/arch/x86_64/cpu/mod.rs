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
//! This module contains code for interrupts, paging, context switches,
//! CPU I/O (from `x86_all`), and reading/writing the x86 control registers.
//!

#[path = "../../x86_all/cpu.rs"] mod cpu_all;

pub mod interrupts;
pub mod paging;
pub mod context;
pub mod control_regs;

pub use self::context::Registers;
pub use self::cpu_all::*;

pub mod segment {

    bitflags! {
        flags Selector: u16 { const RING_0 = 0b00
                            , const RING_1 = 0b01
                            , const RING_2 = 0b10
                            , const RING_3 = 0b11
                            , const GDT    = 0 << 3
                            , const LDT    = 1 << 3
                            }
    }

    impl Selector {
        pub const fn new(index: u16) -> Self {
            Selector { bits: index << 3 }
        }
        pub const fn from_raw(bits: u16) -> Self {
            Selector { bits: bits }
        }
    }
}
