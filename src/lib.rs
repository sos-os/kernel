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
pub mod multiboot;

use arch::cpu;

/// Kernel main loop
///
/// The kernel main loop expects to be passed the address of a valid
/// Multiboot 2 info struct. It's the bootloader's responsibility to ensure
/// that this is passed in the correct register as expected by the calling
/// convention (`edi` on x86). If this isn't there, you can expect to have a
/// bad problem and not go to space today.
#[no_mangle]
pub extern fn kernel_main(multiboot_addr: usize) {
    io::term::CONSOLE.lock().clear();

    println!("Hello from the kernel!");

    // Unpack multiboot tag
    let boot_info = unsafe { multiboot::Info::from(multiboot_addr) };
    let mmap_tag = boot_info.mem_map()
                            .expect("Memory map tag required!");

    println!("Detected memory areas:");
    for a in mmap_tag.entries() {
        println!( "\tstart: 0x{:x}\t\tend: 0x{:x}"
                , a.base, a.length );
    }

    let elf_sections_tag = boot_info.elf64_sections()
                                    .expect("Elf-sections tag required!");

    println!("Detected kernel sections:");
    for section in elf_sections_tag.sections() {
    println!( "\taddr: 0x{:x} \tsize: 0x{:x} \tflags: 0x{:x}"
            , section.address
            , section.length
            , section.flags );
    }

    println!("Intializing interrupts...");
    cpu::interrupts::initialize();
    loop { }
}
