//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # SOS kernel
//! This crate contains the kernel for SOS, the Stupid Operating System.
//!
//! # SOS: the Stupid Operating System
//! SOS is a simple, tiny toy OS implemented in Rust. It targets the `x86`,
//! `x86_64`, and ARM v7 CPU architectures.
//!
//! I'm writing this mostly for fun, to learn more about OS design and kernel
//! hacking, so don't expect anything new or exciting out of this project.
//!
//! SOS is copyright 2015-2017 Eliza Weisman, and is released under the terms
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
#![feature(collections)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr( any(target_arch = "x86_64", target_arch="x86")
           , feature(abi_x86_interrupt))]

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
extern crate cpu;
extern crate elf;
extern crate paging;
extern crate params;
extern crate memory;
extern crate util;

#[macro_use] pub mod io;

pub mod heap;
pub mod arch;
pub mod logger;

use params::InitParams;

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

/// SOS version number
pub const VERSION_STRING: &'static str
    = concat!("Stupid Operating System v", env!("CARGO_PKG_VERSION"));


/// Kernel main loop
pub fn kernel_main() -> ! {
    // let mut a_vec = collections::vec::Vec::<usize>::new();
    // info!(target: "test", "Created a vector in kernel space! {:?}", a_vec);
    // a_vec.push(1);
    // info!(target: "test", "pushed to vec: {:?}", a_vec);
    // a_vec.push(2);
    // info!(target: "test", "pushed to vec: {:?}", a_vec);

    // let mut frame_allocator = frame_alloc::FrameAllocator::new();
    // paging::test_paging(&mut frame_allocator);

    loop { }
}

/// Kernel initialization function called into by architecture-specific init
///
/// Our initialization process essentially looks like this:
///
/// ```text
/// +-------------+
/// | bootloader  |
/// | (multiboot) |
/// +------|------+
/// +------V------+
/// | start.asm   |
/// +------|------+
/// +------|--------------------------------------------------------+
/// |      |           RUST-LAND KERNEL FUNCTIONS                   |
/// |      V                                                        |
/// | arch_init() ----------> kernel_init() --------> kernel_main() |
/// | + collects boot info   + initializes interrupts               |
/// |   from arch-specific   + initializes the heap                 |
/// |   sources              + remaps the kernel into the higher    |
/// | + some CPU-specific      half of the address space            |
/// |   configuration                                               |
/// +---------------------------------------------------------------+
/// ```
pub fn kernel_init(params: &InitParams) {
    kinfoln!("Hello from the kernel!");
    kinfoln!("Got init params: {:#?}", params );
    // -- initialize interrupts ----------------------------------------------
    kinfoln!(dots: " . ", "Initializing interrupts:");
    // TODO: this whole function *may* want to just be made `unsafe`...
    unsafe { arch::interrupts::initialize(); };
    kinfoln!(dots: " . ", target: "Enabling interrupts", "[ OKAY ]");

    // -- initialize the heap ------------------------------------------------
    kinfoln!(dots: " . ", "Preparing to initialize heap.");
    if unsafe { heap::initialize(params) }.is_ok() {
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

    // -- remap the kernel ----------------------------------------------------
    kinfoln!(dots: " . ", "Remapping the kernel:");

    // let frame_allocator = alloc::buddy::BuddyFrameAllocator::new();

    // ::paging::kernel_remap(&params, &frame_allocator);

    kinfoln!( dots: " . ", target: "Remapping the kernel", "[ OKAY ]");

    println!("\n{} {}-bit\n", VERSION_STRING, arch::ARCH_BITS);

    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()
}


/// This fake `main` function exists only to placate `cargo test`.
#[cfg(test)]
fn main() {

}
