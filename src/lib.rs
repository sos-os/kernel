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
#![feature( no_std
          , lang_items)]
#![feature( const_fn
          , core_slice_ext
          , slice_patterns
          )]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate sos_multiboot2 as multiboot;
extern crate sos_alloc as alloc;
#[macro_use] extern crate sos_vga as vga;
#[macro_use] extern crate bitflags;

pub mod arch;
#[macro_use] pub mod io;
pub mod util;
pub mod panic;
pub mod memory;

use arch::cpu;

use alloc::Allocator;
use alloc::simple::SimpleAreaAllocator;

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
    let mmap_tag // Extract the memory map tag from the multiboot info
        = boot_info.mem_map()
                   .expect("Memory map tag required!");

    println!("Detected memory areas:");
    for a in mmap_tag.areas() {
        println!("     start: {:#08x}, end: {:#08x}"
                , a.base, a.length );
    }

    let elf_sections_tag // Extract ELF sections tag from the multiboot info
        = boot_info.elf64_sections()
                   .expect("ELF sections tag required!");

    println!("Detecting kernel ELF sections:");
    // for section in elf_sections_tag.sections() {
    //
    // }

    let kernel_begin
        = elf_sections_tag.sections()
            .map(|s| {
                println!("     address: {:#08x}, size: {:#08x}, flags: {:#08x}"
                        , s.address
                        , s.length
                        , s.flags );
                s.address })
            .min()
            .expect("Could not find kernel start section!\
                    \nSomething is deeply wrong.");

    let mut n_elf_sections = 0;
    let kernel_end
        = elf_sections_tag.sections()
            .map(|s| { n_elf_sections += 1;
                     s.address })
            .max()
            .expect("Could not find kernel end section!\
                    \nSomething is deeply wrong.");

    println!( "Detected {} kernel ELF sections.", n_elf_sections);
    println!( "Kernel begins at {:#x} and ends at {:#x}."
             , kernel_begin, kernel_end );

    let multiboot_end = multiboot_addr + boot_info.length as usize;

    println!( "Multiboot info begins at {:#x} and ends at {:#x}."
             , multiboot_addr, multiboot_end);

    let mut alloc
        = SimpleAreaAllocator::new( kernel_begin as usize
                                  , kernel_end as usize
                                  , multiboot_addr, multiboot_end
                                  , mmap_tag.areas());

    // alloc.allocate(0,0);

    println!( "Created initial allocator." );

    // for i in 0.. {
    //     if let None = alloc.allocate() {
    //         println!("Allocated {} frames", i);
    //         break;
    //     }
    // }
    // println!("Intializing interrupts...");
    // cpu::interrupts::initialize()

    loop { }

}
