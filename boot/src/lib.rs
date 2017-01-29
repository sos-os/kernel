//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Initial 32-bit bootloader for x86_64
#![crate_name = "boot"]
#![feature(asm)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(static_recursion)]
#![feature(linkage)]
#![feature(struct_field_attributes)]
#![feature(stmt_expr_attributes)]
#![no_std]

const TABLE_LENGTH: usize = 512;

type Table = [u64; TABLE_LENGTH];

use core::convert;

macro_rules! set_flags {
    (%$register:ident $( |= $body:expr);+ ) => {
        let mut $register: usize;
        asm!( concat!("mov $0, ", stringify!($register))
            : "=r"($register)
            ::: "intel");
        $($register |= $body;)+
        asm!( concat!("mov ", stringify!($register), ", $0")
            :: "r"($register)
            :: "intel");
    }
}

#[repr(C, packed)]
pub struct Gdt { _null: u64
               , code: u64
               }

#[repr(C, packed)]
pub struct GdtPointer { /// the length of the GDT
                        pub limit: u16
                      , /// pointer to the GDT
                        pub base: &'static Gdt
                      }

impl GdtPointer {
    #[cold] #[inline(always)]
    unsafe fn load (&self) {
        asm!("lgdt ($0)" :: "r"(self) : "memory");
    }
}

impl convert::From<&'static Gdt> for GdtPointer {
    #[cold] #[inline(always)]
    fn from(gdt: &'static Gdt) -> Self {
        use core::mem::size_of_val;
        GdtPointer { limit: size_of_val(gdt) as u16 - 1
                   , base: gdt }
    }
}

#[link_name = ".gdt64"]
#[link_section = ".gdt"]
#[no_mangle]
pub static GDT: Gdt
    = Gdt { _null: 0
          , code: (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)
        //   , data: (1 << 44) | (1 << 47) | (1 << 41)
          };

#[inline(always)]
fn boot_write(s: &[u8]) {
    unsafe {
        use core::ptr::write_volatile;
        let vga_buf = 0xb8000 as *mut u16;
    	for (n, c) in s.iter().enumerate() {
    		write_volatile(vga_buf.offset(n as isize), 0x0200 + *c as u16);
    	}

    }

}

extern "C" {
    static mut pml4_table: Table;
    static mut pdp_table: Table;
    static mut pd_table: Table;
}

#[cold]
#[inline(always)]
#[naked]
unsafe fn create_page_tables() {
    // 3. if everything is okay, create the page tables and start long mode
    const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024; // 2 MiB

    //-- map the PML4 and PDP tables -----------------------------------------
    // recursive map last PML4 entry
    pml4_table[511] = (&pml4_table as *const _ as u64) | 0b11;
    // map first PML4 entry to PDP table
    pml4_table[0] = (&pdp_table as *const _ as u64) | 0b11;
    // map first PDPT entry to PD table
    pdp_table[0] = (&pd_table as *const _ as u64) | 0b11;

    boot_write(b"3.1");

    //-- map the PD table ----------------------------------------------------
    for (number, entry) in pd_table.iter_mut().enumerate() {
        // set each PD entry equal to the start address of the page (the page
        // number times the page's size)
        let addr = number as u64 * HUGE_PAGE_SIZE;
        // with the appropriate flags (present + writable + huge)
        // TODO: do we want to do this using bitflags, or is that too
        //       heavyweight for the boot module?
        //          - eliza, 1/23/2017
        *entry = addr | 0b10000011;
    }

    boot_write(b"3.2");
}
#[cold]
#[inline(always)]
#[naked]
unsafe fn set_long_mode() {
    // load PML4 addr to cr3
    asm!("mov cr3, $0" :: "r"(&pml4_table) :: "intel");
    boot_write(b"3.3");

    // // enable PAE flag in cr4
    set_flags!(%cr4 |= 1 << 5 );
    boot_write(b"3.4");

    // set the long mode bit in EFER MSR (model specific register)
    asm!( "mov   ecx, 0xC0000080
           rdmsr
           or    eax, 1 << 8
           wrmsr"
        :::: "intel");
    boot_write(b"3.5");

    // enable paging in cr0
    set_flags!(%cr0 |= 1 << 31;
                    |= 1 << 16 );
}

/// Test whether or not this system supports Multiboot 2
#[cold]
#[inline(always)]
unsafe fn is_multiboot_supported() -> bool {
    let eax: usize;
    asm!("mov eax, $0" : "=r"(eax) ::: "intel");
    eax == 0x36d76289
}


#[cold]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
    boot_write(b"0");
    asm!("cli");

    // 1. Move Multiboot info pointer to edi
    asm!("mov edi, ebx" :::: "intel");
    boot_write(b"1");

    // 2. make sure the system supports SOS
    // TODO: port this from boot.asm
    if !is_multiboot_supported() {
        loop { boot_write(b"ERROR: multiboot not supported!"); }
    }
    boot_write(b"2");

    create_page_tables();
    set_long_mode();

    // 4. load the 64-bit GDT
    GdtPointer::from(&GDT).load();
    boot_write(b"4");

    // 6. jump to the 64-bit boot subroutine.
    asm!("ljmpl $$8, $$long_mode_init");

}
