//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! `x86_64` architecture-specific implementation.
// pub mod cpu;
pub mod drivers;
pub mod interrupts;
// pub mod memory;

#[path = "../x86_all/bda.rs"] pub mod bda;
#[path = "../x86_all/multiboot2.rs"] pub mod multiboot2;

pub const ARCH_BITS: u8 = 64;

use memory::PAddr;

/// Entry point for architecture-specific kernel init
///
/// This expects to be passed the address of a valid
/// Multiboot 2 info struct. It's the bootloader's responsibility to ensure
/// that this is passed in the correct register as expected by the calling
/// convention (`edi` on x86). If this isn't there, you can expect to have a
/// bad problem and not go to space today.
#[no_mangle]
pub extern "C" fn arch_init(multiboot_addr: PAddr) {
    use cpu::{control_regs, msr};
    use elf;
    use memory::{PAddr, Page, PhysicalPage};
    use params::InitParams;
    use ::kernel_init;

    kinfoln!(dots: " . ", "Beginning `arch_init()` for x86_64");

    ::io::term::CONSOLE.lock().clear();
    ::logger::initialize()
        .expect("Could not initialize logger!");

    // -- Unpack multiboot tag -----------------------------------------------
    // try to interpret the structure at the multiboot address as a multiboot
    // info struct. if it's invalid, fail.
    let boot_info
        = unsafe { multiboot2::Info::from(multiboot_addr)
                    .expect("Could not unpack multiboot2 information!") };

    // Extract the memory map tag from the multiboot info
    let mmap_tag
        = boot_info.mem_map()
                   .expect("Memory map tag required!");

    kinfoln!(dots: " . ", "Detected memory areas:");
    for a in mmap_tag.areas() {
        kinfoln!( dots: " . . ", "start: {:#08x}, end: {:#08x}"
                , a.base, a.length );
        // TODO: add these to a list of memory areas?
        //       - eliza, 1/23/2017
    }

    // Extract ELF sections tag from the multiboot info
    let elf_sections_tag
        = boot_info.elf_sections()
                   .expect("ELF sections tag required!");

    kinfoln!(dots: " . ", "Detecting kernel ELF sections:");

    // Extract kernel ELF sections from  multiboot info
    let mut n_elf_sections = 0;
    let kernel_begin
        = elf_sections_tag.sections()
            .inspect(|s| {
                kinfoln!( dots: " . . "
                        , "address: {:#08x}, size: {:#08x}, flags: {:#08x}"
                        , s.addr()
                        , s.length()
                        , s.flags() );
                n_elf_sections += 1;
                })
            // the ELF section with the lowest address is the kernel start
            // section
            .min_by_key(elf::Section::addr)
            .expect("Could not find kernel start section!\
                    \nSomething is deeply wrong.");


    let kernel_end
        = elf_sections_tag.sections()
            .max_by_key(elf::Section::addr)
            .expect("Could not find kernel end section!\
                    \nSomething is deeply wrong.");

    kinfoln!( dots: " . ", "Detected {} kernel ELF sections.", n_elf_sections);
    kinfoln!( dots: " . . ", "Kernel begins at {:#p} and ends at {:#p}."
            , kernel_begin.addr(), kernel_end.addr() );

    let multiboot_end = multiboot_addr + boot_info.length as u64;

    kinfoln!( dots: " . . ", "Multiboot info begins at {:#x} and ends at {:#x}."
            , multiboot_addr, multiboot_end);

    let params = InitParams { multiboot_start: Some(multiboot_addr)
                            , multiboot_end: Some(multiboot_end)
                            , ..Default::default()
                            };

     // -- enable flags needed for paging ------------------------------------
     unsafe {
         control_regs::cr0::enable_write_protect(true);
         kinfoln!(dots: " . ", "Page write protect ENABED" );

         msr::enable_nxe();
         kinfoln!(dots: " . ", "Page no execute bit ENABLED");
     }

    kinfoln!(dots: " . ", "Transferring to `kernel_init()`.");
    kernel_init(params);
}
