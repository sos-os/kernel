//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! x86_64 architecture-specific implementation.
pub mod cpu;
pub mod drivers;
pub mod memory;

use memory::PAddr;
use multiboot2;


/// Entry point for architecture-specific kernel init
#[no_mangle]
pub extern "C" fn arch_init(multiboot_addr: PAddr) {
    // -- Unpack multiboot tag ------------------------------------------------
    let boot_info = unsafe {
        multiboot2::Info::from(multiboot_addr)
            .expect("Could not unpack multiboot2 information!")
    };
}
