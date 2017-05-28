//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! `x86_64` architecture-specific implementation.
// pub mod cpu;
pub mod drivers;
pub mod interrupts;

#[path = "../x86_all/bda.rs"] pub mod bda;
#[path = "../x86_all/multiboot2.rs"] pub mod multiboot2;

pub const ARCH_BITS: u8 = 64;

extern {
    // TODO: It would be really nice if there was a less ugly way of doing
    // this... (read: after the Revolution when we add memory regions to the
    // heap programmatically.)
    #[link_name = "heap_base_addr"]
    #[linkage = "external"]
    pub static HEAP_BASE: *mut u8;
    #[link_name = "heap_top_addr"]
    #[linkage = "external"]
    pub static HEAP_TOP: *mut u8;
    // Of course, we will still need to export the kernel stack addresses like
    // this, but it would be nice if they could be, i dont know, not mut u8s
    // pointers, like God intended.
    #[link_name = "stack_base"]
    pub static STACK_BASE: *mut u8;
    #[link_name = "stack_top"]
    pub static STACK_TOP: *mut u8;
}

use memory::PAddr;

/// Trampoline to ensure we have a correct stack frame for calling [`arch_init`]
///
/// I have no idea why this works, but it does.
///
/// [`arch_init`]: fn.arch_init
#[naked]
#[no_mangle]
pub unsafe extern "C" fn long_mode_init() {
    asm!("movabsq $$(stack_top), %rsp");
    asm!("mov ax, 0
          mov ss, ax
          mov ds, ax
          mov es, ax
          mov fs, ax
          mov gs, ax
          call arch_init"
        :::: "intel");

}

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
    use params::{InitParams, mem};

    kinfoln!(dots: " . ", "Beginning `arch_init()` for x86_64");

    ::io::term::CONSOLE.lock().clear();
    ::logger::initialize()
        .expect("Could not initialize logger!");


    // -- Unpack multiboot tag ------------------------------------------------
    kinfoln!( dots: " . "
            , "trying to unpack multiboot info at {:#p}"
            , multiboot_addr);

    // try to interpret the structure at the multiboot address as a multiboot
    // info struct. if it's invalid, fail.
    let boot_info
        = unsafe { multiboot2::Info::from(multiboot_addr)
                    .expect("Could not unpack multiboot2 information!") };

    // Extract ELF sections tag from the multiboot info
    let elf_sections_tag
        = boot_info.elf_sections()
                   .expect("ELF sections tag required!");

    kinfoln!(dots: " . ", "Detecting kernel ELF sections:");

    // Extract kernel ELF sections from  multiboot info
    let mut n_elf_sections = 0;

    let kernel_begin
        = elf_sections_tag.sections()
            // .filter(|s| s.is_allocated())
            .map(|s| {
                kinfoln!( dots: " . . ", "{}", s );
                kinfoln!( dots: " . . . ", "flags: [ {:?} ]", s.flags());
                s.address() })
            .min()
            .expect("Could not find kernel start section!\
                    \nSomething is deeply wrong.");


    let kernel_end
        = elf_sections_tag.sections()
            // .filter(|s| s.is_allocated())
            .map(|s| { n_elf_sections += 1; s.end_address() })
            .max()
            .expect("Could not find kernel end section!\
                    \nSomething is deeply wrong.");

    kinfoln!( dots: " . ", "Detected {} kernel ELF sections.", n_elf_sections);
    kinfoln!( dots: " . . ", "Kernel begins at {:#p} and ends at {:#p}."
            , kernel_begin, kernel_end );

    let multiboot_end = multiboot_addr + boot_info.length as u64;

    kinfoln!( dots: " . . ", "Multiboot info begins at {:#x} and ends at {:#x}."
            , multiboot_addr, multiboot_end);

    let mut params = InitParams { kernel_base: kernel_begin
                            , kernel_top: kernel_end
                            , multiboot_start: Some(multiboot_addr)
                            , multiboot_end: Some(multiboot_end)
                            , heap_base: unsafe { PAddr::from(HEAP_BASE) }
                            , heap_top: unsafe { PAddr::from(HEAP_TOP) }
                            , stack_base: unsafe { PAddr::from(STACK_BASE) }
                            , stack_top: unsafe { PAddr::from(STACK_TOP) }
                            , elf_sections: Some(elf_sections_tag.sections())
                            , ..Default::default()
                        };

    // Extract the memory map tag from the multiboot info
    let mem_map = boot_info.mem_map()
                           .expect("Memory map tag required!");

    kinfoln!(dots: " . ", "Detected memory areas:");
    for area in mem_map {
        kinfoln!( dots: " . . ", "{}", area);
        let a: mem::Area = area.into();
        if a.is_usable == true { params.mem_map.push(a); }
    }

     //-- enable flags needed for paging ------------------------------------
     unsafe {
        //  control_regs::cr0::enable_write_protect(true);
        //  kinfoln!(dots: " . ", "Page write protect ENABED" );

        let efer = msr::read(msr::IA32_EFER);
        trace!("EFER = {:#x}", efer);
        msr::write(msr::IA32_EFER, efer | (1 << 11));
        let efer = msr::read(msr::IA32_EFER);
        trace!("EFER = {:#x}", efer);
        kinfoln!(dots: " . ", "Page no execute bit ENABLED");
     }

    kinfoln!(dots: " . ", "Transferring to `kernel_init()`.");
    ::kernel_init(&params);
}
