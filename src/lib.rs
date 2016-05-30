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
//! I'm writing this mostly for fun, to learn more about OS design and kernel
//! hacking, so don't expect anything new or exciting out of this project.

#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]
#![feature(core_intrinsics)]
#![feature( lang_items, asm )]
#![feature( const_fn
          , slice_patterns
          , associated_consts
          , unique )]
#![feature(collections)]
#![feature(question_mark)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![no_std]

extern crate collections;
extern crate rlibc;
extern crate spin;

extern crate sos_alloc as alloc;

#[macro_use] extern crate sos_vga as vga;
#[macro_use] extern crate bitflags;

#[macro_use] pub mod memory;
#[macro_use] pub mod io;

pub mod util;
pub mod panic;
pub mod multiboot2;
pub mod elf;
pub mod arch;

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
                , memory::heap_base_addr()
                , memory::heap_top_addr());
    };
    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()
}
