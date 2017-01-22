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
    // use multiboot2;
    use cpu::{control_regs, msr};
    use alloc::buddy;
    use memory::{PAddr, Page, PhysicalPage};
    use params::InitParams;
    use ::kernel_init;

    ::io::term::CONSOLE.lock().clear();
    ::logger::initialize()
        .expect("Could not initialize logger!");

    // -- Unpack multiboot tag -----------------------------------------------
    let boot_info
        = unsafe { multiboot2::Info::from(multiboot_addr)
                    .expect("Could not unpack multiboot2 information!") };

    let mmap_tag // Extract the memory map tag from the multiboot info
        = boot_info.mem_map()
                   .expect("Memory map tag required!");

    kinfoln!(dots: " . ", "Detected memory areas:");
    for a in mmap_tag.areas() {
        kinfoln!( dots: " . . ", "start: {:#08x}, end: {:#08x}"
                , a.base, a.length );
    }

    let elf_sections_tag // Extract ELF sections tag from the multiboot info
        = boot_info.elf_sections()
                   .expect("ELF sections tag required!");

    kinfoln!(dots: " . ", "Detecting kernel ELF sections:");

    let kernel_begin    // Extract kernel ELF sections from  multiboot info
        = elf_sections_tag.sections()
            .map(|s| {
                kinfoln!( dots: " . . "
                        , "address: {:#08x}, size: {:#08x}, flags: {:#08x}"
                        , s.addr()
                        , s.length()
                        , s.flags() );
                s.addr() })
            .min()
            .expect("Could not find kernel start section!\
                    \nSomething is deeply wrong.");

    let mut n_elf_sections = 0;
    let kernel_end
        = elf_sections_tag.sections()
            .map(|s| { n_elf_sections += 1; s.addr() })
            .max()
            .expect("Could not find kernel end section!\
                    \nSomething is deeply wrong.");

    kinfoln!( dots: " . ", "Detected {} kernel ELF sections.", n_elf_sections);
    kinfoln!( dots: " . . ", "Kernel begins at {:#x} and ends at {:#x}."
            , kernel_begin, kernel_end );

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

     println!(" . . Preparing to initialize heap. ");
     // -- initialize the heap -----------------------------------------------
     // TODO: I think this is in kernel_init now – not sure which is
     //       Correcter. Should figure that out.
     //          - eliza, 1/21/2017
     let heap_base
        = PhysicalPage::containing_addr(PAddr::from(multiboot_addr + boot_info.length as u64)).base();

    //  unsafe {
    //      buddy::system::init_heap(heap_base.as_mut_ptr(), ::memory::HEAP_SIZE);
    //      println!( "{:<38}{:>40}\n \
    //                  . . Heap begins at {:#x} and ends at {:#x}"
    //              , " . Intializing heap"
    //             // , ::memory::init_heap(heap_base.as_mut_ptr())
    //             //            .unwrap_or("[ FAIL ]")
    //              , "[ OKAY ]"
    //              , heap_base
    //              , heap_base + ::memory::HEAP_SIZE as u64);
    //  };

    // -- remap the kernel ----------------------------------------------------
    // TODO: should this happen _after_ non-arch kernel initialization?
    //          - eliza, 1/22/2017
    kinfoln!(dots: " . ", "Remapping the kernel:");

    let frame_allocator = buddy::BuddyFrameAllocator::new();
    ::paging::kernel_remap(&params, &frame_allocator);

    kinfoln!( dots: " . ", target: "Remapping the kernel", "[ OKAY ]");

    kernel_init(params);
}
