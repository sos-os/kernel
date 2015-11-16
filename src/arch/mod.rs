//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific implementation

// 64-bit x86_64 (long mode)
#[cfg(target_arch="x86_64")] mod x86_64;
#[cfg(target_arch="x86_64")] pub use self::x86_64::{ cpu, drivers };

// 32-bit x86 (protected mode)
// TODO: NYI
#[cfg(target_arch = "x86")] mod x86;
#[cfg(target_arch = "x86")] pub use self::x86::{ cpu, drivers };

// ARM v7
// TODO: NYI
#[cfg(target_arch = "armv7")] mod armv7;
#[cfg(target_arch = "armv7")] pub use self::x86::{ cpu, drivers };
