//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! `x86` architecture-specific implementation.
//!
//! This module contains code for `x86` 32-bit protected-mode systems.
pub mod cpu;
pub mod memory;
// pub mod keyboard;

pub const ARCH_BITS: u8 = 32;
