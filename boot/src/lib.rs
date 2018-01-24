//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2016-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # SOS x86_64 bootstrap
//!
//! This crate contains code for the 32-bit protected mode boot routine for
//! x86_64 CPUs. When we boot up an x86_64 system using GRUB & multiboot, we
//! end up with the CPU running in 32-bit 'protected mode', rather than 64-bit
//! 'long mode'. Before we can jump into long mode, we have to perform a
//! handful of setup tasks in protected mode.
//!
//! This boot routine lives in a separate crate, with a 32-bit `target.json`.
//! When building SOS for x86_64, we have to compile this crate separately and
//! then link the resulting 32-bit object with the rest of the 64-bit kernel
//! object.
//!
//! This boot crate is only necessary for x86_64. For other architectures, all
//! boot code can live in the main kernel crate.
//!
//! For more information, refer to:
//! + the [intermezzOS book]
//! + the [OSDev Wiki] article on long mode
//! + Philipp Oppermann's [blog post]
//!
//! [intermezzOS book]: http://intermezzos.github.io/book/transitioning-to-long-mode.html
//! [OSDev Wiki]: http://wiki.osdev.org/Long_Mode#Long_Mode
//! [blog post]: http://os.phil-opp.com/entering-longmode.html

#![crate_name = "boot32"]
#![feature(asm)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(linkage)]
#![feature(struct_field_attributes)]
#![feature(stmt_expr_attributes)]
#![no_std]

extern crate rlibc;

const TABLE_LENGTH: usize = 512;
/// The size of a "huge" page
const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024; // 2 MiB
/// Page table entry flags for a page that is present and writable.
const ENTRY_FLAGS_PW: u64 = 0b11;

/// A page table is an array of page table entries.
type Table = [TableEntry; TABLE_LENGTH];

trait PageTable: Sized {
    /// Install this page table as the top-level page table.
    #[inline(always)]
    unsafe fn set_top_level(&'static self) {
        asm!("mov cr3, $0" :: "r"(self) :: "intel");
    }
}

impl PageTable for Table { }

/// A page table entry is a 64-bit unsigned integer.
///
/// We represent this as a [newtype] rather than a type alias, because we want
/// to implement specific functionality on it.
///
/// [newtype]: https://aturon.github.io/features/types/newtype.html
#[repr(C)]
struct TableEntry(u64);

impl TableEntry {
    /// Set this table entry to map to a lower-level page table.
    // TODO: use a marker type to ensure this always maps to a _lower-level_
    //       table?
    #[inline(always)]
    unsafe fn map_to_table(&mut self, to: &'static Table) {
        *self = TableEntry(to as *const _ as u64 | ENTRY_FLAGS_PW);
    }

    /// Set this table entry to map to a huge page with the given number.
    #[inline(always)]
    unsafe fn map_to_page(&mut self, number: usize) {
        // Page table entry flags for a page that is huge, present, and writable.
        const ENTRY_FLAGS_HUGE: u64 = 0b10000011;
        // the start address is the page number times the page's size
        let addr = number as u64 * HUGE_PAGE_SIZE;
        *self = TableEntry(addr | ENTRY_FLAGS_HUGE);
    }

}

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
    //-- map the PML4 and PDP tables -----------------------------------------
    // recursive map last PML4 entry
    pml4_table[511].map_to_table(&pml4_table);
    // map first PML4 entry to PDP table
    pml4_table[0].map_to_table(&pdp_table);
    // map first PDPT entry to PD table
    pdp_table[0].map_to_table(&pd_table);

    boot_write(b"3.1");

    //-- map the PD table ----------------------------------------------------
    for (page_number, entry) in pd_table.iter_mut().enumerate() {
        entry.map_to_page(page_number);
    }

    boot_write(b"3.2");
}

#[cold]
#[inline(always)]
unsafe fn set_long_mode() {
    // load PML4 addr to cr3
    &pml4_table.set_top_level();
    boot_write(b"3.3");

    // // enable PAE flag in cr4
    // set_flags!(%cr4 |= 1 << 5 );
    asm!("mov eax, cr4
          or eax, 1 << 5
          mov cr4, eax" ::: "memory" : "intel", "volatile");
    boot_write(b"3.4");

    // set the long mode bit in EFER MSR (model specific register)
    asm!( "mov   ecx, 0xC0000080
           rdmsr
           or    eax, 1 << 8
           wrmsr"
        :::"memory": "intel", "volatile");
    boot_write(b"3.5");

    // enable paging in cr0
    // set_flags!(%cr0 |= 1 << 31;
    //                 |= 1 << 16 );

    asm!("mov eax, cr0
          or eax, 0x80000000
          mov cr0, eax" :::"memory": "intel", "volatile");
    boot_write(b"3.6")
}

/// Test whether or not this system supports Multiboot 2
#[cold]
#[inline(always)]
unsafe fn is_multiboot_supported() -> bool {
    const MULTIBOOT_MAGIC: usize = 0x36d76289;
    let eax: usize;
    asm!("mov eax, $0" : "=r"(eax) ::: "intel");
    eax == MULTIBOOT_MAGIC
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

    // 3. if everything is okay, create the page tables and start long mode
    create_page_tables();
    set_long_mode();

    // 4. load the 64-bit GDT
    GdtPointer::from(&GDT).load();
    boot_write(b"4");

    // 6. jump to the 64-bit boot subroutine.
    asm!("ljmpl $$8, $$long_mode_init");

}

#[cold]
#[lang = "panic_fmt"]
fn panic_fmt() -> ! {
    loop {
        boot_write(b"panic! panic! panic!");
    }
}