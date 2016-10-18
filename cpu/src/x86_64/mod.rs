//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the `x86_64` CPU.
//!
//! This module contains code for interrupts, paging, context switches,
//! CPU I/O (from `x86_all`), and reading/writing the x86 control registers.
//!

#[path = "../x86_all/mod.rs"] mod cpu_all;

pub mod context;
pub mod task;

pub use self::context::Registers;
pub use self::cpu_all::*;
