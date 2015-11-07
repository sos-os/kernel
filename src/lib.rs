//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS: the Stupid Operating System
//!
//! SOS is a simple, tiny toy OS implemented in Rust.
//!
//! I'm writing this mostly for fun, to learn more about OS design and kernel //! hacking, so don't expect anything new or exciting out of this project.

#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]
#![feature(asm)]
#![feature(no_std, lang_items)]
#![feature(const_fn, unique, core_str_ext, core_slice_ext)]
#![feature(slice_patterns)]
#![no_std]

extern crate rlibc;
extern crate spin;

pub mod arch;
#[macro_use]
pub mod io;
pub mod util;
pub mod panic;

/// Kernel main loop
#[no_mangle]
pub extern fn kernel_main() {
    println!("Help, I'm trapped inside a kernel factory!");
    loop { }
}
