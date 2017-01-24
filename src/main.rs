//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # SOS kernel
//! This crate contains the kernel for SOS, the Stupid Operating System.
//!
//! # SOS: the Stupid Operating System
//! SOS is a simple, tiny toy OS implemented in Rust. It targets the x86,
//! x86_64, and ARM v7 CPU architectures.
//!
//! I'm writing this mostly for fun, to learn more about OS design and kernel
//! hacking, so don't expect anything new or exciting out of this project.
//!
//! SOS is copyright 2015-2016 Eliza Weisman, and is released under the terms
//! of the MIT license.

#![crate_name = "sos_kernel"]

#![doc(html_root_url = "https://hawkw.github.io/sos-kernel/")]

#![feature( lang_items, asm, naked_functions )]
#![feature( linkage )]
#![feature( const_fn
          , slice_patterns
          , associated_consts
          , type_ascription
          , custom_derive )]
#![feature( collections )]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![no_std]
#![cfg_attr(not(test), no_main)]

// -- non-SOS dependencies --------------------------------------------------
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;

extern crate collections;
extern crate rlibc;
extern crate spin;

// -- SOS dependencies ------------------------------------------------------
#[macro_use] extern crate vga;

extern crate alloc;
#[macro_use] extern crate cpu;
extern crate elf;
extern crate util;
extern crate memory;

#[macro_use] pub mod io;

pub mod heap;
pub mod params;
pub mod arch;
pub mod logger;

/// SOS version number
pub const VERSION_STRING: &'static str
    = concat!("Stupid Operating System v", env!("CARGO_PKG_VERSION"));

use params::InitParams;

/// Kernel main loop
pub fn kernel_main() -> ! {
    let mut a_vec = collections::vec::Vec::<usize>::new();
    info!(target: "test", "Created a vector in kernel space! {:?}", a_vec);
    a_vec.push(1);
    info!(target: "test", "pushed to vec: {:?}", a_vec);
    a_vec.push(2);
    info!(target: "test", "pushed to vec: {:?}", a_vec);

    loop { }
}

/// Kernel initialization function called from ASM
///
/// The kernel main loop expects to be passed the address of a valid
/// Multiboot 2 info struct. It's the bootloader's responsibility to ensure
/// that this is passed in the correct register as expected by the calling
/// convention (`edi` on x86). If this isn't there, you can expect to have a
/// bad problem and not go to space today.
//  TODO: since multiboot2 is x86-specific, this needs to move to `arch`.
//  we then want the kernel entry point to be `arch_init`. we can then
//  call into `kernel_init`.
pub fn kernel_init(params: InitParams) {
    kinfoln!("Hello from the kernel!");

    // -- initialize interrupts ----------------------------------------------
    kinfoln!(dots: " . ", "Initializing interrupts:");
    unsafe {
        arch::interrupts::initialize();
    };
    kinfoln!(dots: " . ", target: "Enabling interrupts", "[ OKAY ]");

    // -- initialize the heap ------------------------------------------------

    if let Ok(_) =  unsafe { heap::initialize(&params) } {
        kinfoln!( dots: " . ", target: "Intializing heap"
                , "[ OKAY ]"
                );
        kinfoln!( dots: " . . "
                , "Heap begins at {:#x} and ends at {:#x}"
                , params.heap_base, params.heap_top);
    } else {
        kinfoln!( dots: " . ", target: "Intializing heap"
                , "[ FAIL ]"
                );
    }

    println!("\n{} {}-bit\n", VERSION_STRING, arch::ARCH_BITS);
    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()

}


/// This fake `main` function exists only to placate `cargo test`.
#[cfg(test)]
fn main() {

}
