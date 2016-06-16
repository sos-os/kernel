//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Hawk Weisman
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
//! SOS is copyright 2015-2016 Hawk Weisman, and is released under the terms
//! of the MIT license.

#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]

#![doc(html_root_url = "https://hawkw.github.io/sos-kernel/")]

#![feature(core_intrinsics)]
#![feature( lang_items, asm, naked_functions )]
#![feature( const_fn
          , slice_patterns
          , associated_consts
          , unique
          , type_ascription
          , custom_derive )]
#![feature(collections)]
#![feature(question_mark)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![no_std]

// -- non-SOS dependencies --------------------------------------------------
extern crate collections;
extern crate rlibc;
extern crate spin;

#[macro_use] extern crate bitflags;
#[macro_use] extern crate custom_derive;

// -- SOS dependencies ------------------------------------------------------
extern crate sos_alloc as alloc;

#[macro_use] extern crate sos_vga as vga;

#[macro_use] pub mod memory;
#[macro_use] pub mod io;

pub mod util;
pub mod multiboot2;
pub mod elf;
pub mod arch;

// Since the test module contains lang items, it can't be compiled when
// running tests.
#[cfg(not(test))] pub mod panic;

use arch::cpu;
use memory::PAddr;

#[macro_use]
macro_rules! init_log {
    (f$dots:expr, $task:expr, $msg:expr) => (
        println!( "{task:<40}{res:>38}\n{msg:>.width$}"
                , task = format!("{:>.width$}", $task, width = $dots)
                , res = "[ FAIL ]"
                , msg = $msg
                , width = $dots + 1
                )
    );
    (fail: $dots:expr, $task:expr) => (
            println!( "{task:<40}{res:>38}"
                    , task = format!("{:>.width$}", $task, width = $dots)
                    , res = "[ FAIL ]"
                    )
    );
    (okay: $dots:expr, $task:expr, $msg:expr) => (
        println!( "{task:<40}{res:>38}\n{msg:>.width$}"
                , task = format!("{:>.width$}", $task, width = $dots)
                , res = "[ OKAY ]"
                , msg = $msg
                , width = $dots + 1
                )
    );
    (okay: $dots:expr, $task:expr) => (
            println!( "{task:<40}{res:>38}"
                    , task = format!("{:>.width$}", $task, width = $dots)
                    , res = "[ OKAY ]"
                    )
    );
}

macro_rules! init_try {
    ($dots:expr, $task:expr, $result:expr) => (
        match $result {
            Ok(value) => {
                println!( "{task:<40}{res:>38}"
                        , task = format!("{:>.width$}", $task, width = $dots)
                        , res = "[ OKAY ]"
                        );
                value
            }
          , Err(why) => {
                println!( "{task:<40}{res:>38}\n\n{msg:>.width$}"
                        , task = format!("{:>.width$}", $task, width = $dots)
                        , res = "[ FAIL ]"
                        , msg = why
                        , width = $dots + 1
                        );
                return $expr
            }
        }
    )
}

/// Kernel main loop
pub fn kernel_main() {
    let mut a_vec = collections::vec::Vec::<usize>::new();
    println!( "TEST: Created a vector in kernel space! {:?}", a_vec);
    a_vec.push(1);
    println!( "TEST: pushed to vec: {:?}", a_vec);
    a_vec.push(2);
    println!( "TEST: pushed to vec: {:?}", a_vec);
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
#[no_mangle]
pub extern fn kernel_start(multiboot_addr: PAddr) {
    io::term::CONSOLE.lock().clear();

    println!("Hello from the kernel!");

    // -- jump to architecture-specific init ---------------------------------
    arch::arch_init(multiboot_addr);

    // -- initialize interrupts ----------------------------------------------
    unsafe {
        println!(" . Enabling interrupts:");
        cpu::interrupts::initialize();
        println!("{:<38}{:>40}", " . Enabling interrupts", "[ OKAY ]");
    };

    // -- initialize the heap ------------------------------------------------
    unsafe {
        println!( "{:<38}{:>40}\n \
                    . . Heap begins at {:#x} and ends at {:#x}"
                , " . Intializing heap"
                , memory::init_heap().unwrap_or("[ FAIL ]")
                , memory::HEAP_BASE
                , memory::HEAP_TOP);
    };
    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()
}
