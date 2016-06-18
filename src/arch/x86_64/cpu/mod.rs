//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the `x86_64` CPU.
//!
//! This module contains code for interrupts, paging, context switches,
//! CPU I/O (from `x86_all`), and reading/writing the x86 control registers.
//!

#[path = "../../x86_all/cpu/mod.rs"] mod cpu_all;

/// 64-bit Interrupt Descriptor Table implementation.
///
/// Refer to section 6.10 of the _Intel® 64 and IA-32 Architectures
/// Software Developer’s Manual_ for more information.
#[path = "../../x86_all/interrupts/mod.rs"]
pub mod interrupts;

pub mod context;
pub mod task;
pub mod msr;

pub use self::context::Registers;
pub use self::cpu_all::*;
