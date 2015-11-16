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
