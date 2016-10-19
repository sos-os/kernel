//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
// 64-bit x86_64 (long mode)
#[cfg(target_arch="x86_64")] mod x86_64;
#[cfg(target_arch="x86_64")] pub use self::x86_64::*;

// 32-bit x86 (protected mode)
// TODO: NYI
#[cfg(target_arch = "x86")] mod x86;
#[cfg(target_arch = "x86")] pub use self::x86::*;
