//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-specific implementation.
//!
//! This module consists of a number of modules containing
//! architecture-specific code for each targeted architecture. The `arch`
//! module uses conditional compilation to re-export the implementation for
//! which the kernel is currently being compiled.
//!
//! In order for the rest of the kernel to work properly, an
//! architecture-specific implementation module should define a number of
//! specific items. If these are not defined, the platform-independant kernel
//! implementation cannot function properly.
//!
//! Please note that currently only the architecture-specific implementation
//! for `x86_64` (long mode) is implemented. The `armv7` and `x86` (protected
//! mode) modules are currently much less complete.

// 64-bit x86_64 (long mode)
#[cfg(target_arch="x86_64")] mod x86_64;
#[cfg(target_arch="x86_64")] pub use self::x86_64::*;

// 32-bit x86 (protected mode)
// TODO: NYI
#[cfg(target_arch = "x86")] mod x86;
#[cfg(target_arch = "x86")] pub use self::x86::*;

// ARM v7
// TODO: NYI
#[cfg(target_arch = "armv7")] mod armv7;
#[cfg(target_arch = "armv7")] pub use self::x86::*;
