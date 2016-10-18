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
// #![crate_type = "staticlib"]

#![doc(html_root_url = "https://hawkw.github.io/sos-kernel/")]

#![feature( core_intrinsics )]
#![feature( lang_items, asm, naked_functions )]
#![feature( linkage )]
#![feature( const_fn
          , slice_patterns
          , associated_consts
          , type_ascription
          , custom_derive )]
#![feature( collections )]
#![feature( question_mark )]
// #![warn( missing_docs )]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![no_std]
#![cfg_attr(not(test), no_main)]

// -- non-SOS dependencies --------------------------------------------------
extern crate collections;
extern crate rlibc;
extern crate spin;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate bitflags;
// #[macro_use] extern crate custom_derive;

// -- SOS dependencies ------------------------------------------------------
#[macro_use] extern crate vga;
#[macro_use] extern crate cpu;
extern crate util;
extern crate alloc;
extern crate memory;

// #[macro_use] pub mod macros;
// #[macro_use] pub mod memory;
#[macro_use] pub mod io;

pub mod heap;
pub mod params;
// pub mod multiboot2;
pub mod elf;
pub mod arch;

/// SOS version number
pub const VERSION_STRING: &'static str
    = concat!("Stupid Operating System v", env!("CARGO_PKG_VERSION"));

// pub const BUILD_STRING: &'static str
//     = concat!("Built with ", env!("RUST_VERSION"));
//
// Since the test module contains lang items, it can't be compiled when
// running tests.
// #[cfg(not(test))] pub mod panic;

// use arch::cpu;
use params::InitParams;
// use core::fmt::Write;

//
// #[cfg(not(debug_assertions))]
// #[macro_use]
// macro_rules! log {
//     ($descriptor:expr, $dots:expr, $msg:expr) => (
//         // arch::drivers::serial::COM1.map(|com1|
//         //     write!(com1.lock(), "{}: {}" descriptor, msg)
//         // );
//         println!("{}{}", $dots, $msg)
//     )
// }
//

/// Kernel main loop
pub fn kernel_main() -> ! {
    let mut a_vec = collections::vec::Vec::<usize>::new();
    println!( "TEST: Created a vector in kernel space! {:?}", a_vec);
    a_vec.push(1);
    println!( "TEST: pushed to vec: {:?}", a_vec);
    a_vec.push(2);
    println!( "TEST: pushed to vec: {:?}", a_vec);
    // loop {
    //     unsafe { asm!("int $0" :: "N" (0x80)) };
    //     println!("Test interrupt okay");
    // }
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
    infoln!("Hello from the kernel!");

    // -- initialize interrupts ----------------------------------------------
    infoln!(dots: " . ", "Initializing interrupts:");
    unsafe {
        arch::interrupts::initialize();
    };

    infoln!(dots: " . ", "Enabling interrupts", status: "[ OKAY ]");

    // -- initialize the heap ------------------------------------------------
    unsafe {
        infoln!( dots: " . ", "Intializing heap"
             , status: heap::initialize(&params).unwrap_or("[ FAIL ]")
             );
        infoln!( dots: " . . "
             , "Heap begins at {:#x} and ends at {:#x}"
             , params.heap_base, params.heap_top);
    };

    println!("\n{} {}-bit\n", VERSION_STRING, arch::ARCH_BITS);
    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()

}


/// This fake `main` function exists only to placate `cargo test`.
#[cfg(test)]
fn main() {

}
