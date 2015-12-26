//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86 architecture-specific implementation.
//!
//! This module contains code that is portable between x86 32-bit
//! protected-mode systems and x86_64 64-bit long mode systems.
pub mod cpu;
pub mod interrupts;
// pub mod keyboard;
