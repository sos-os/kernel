//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86 architecture-specific implementation.

#[path = "../x86_all/vga.rs"]
pub mod vga;
pub mod cpu;
pub mod interrupts;
// pub mod keyboard;
