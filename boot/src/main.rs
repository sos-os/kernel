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
#![crate_type = "staticlib"]
#![feature(asm)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

const TABLE_LENGTH: usize = 512;

type Table = [u64; TABLE_LENGTH];

use core::fmt::Arguments;

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: Arguments, file: &'static str, line: u32) -> ! {
	loop {}
}

extern {
    static mut pml4_table: Table;
    static mut pdp_table: Table;
    static mut pd_table: Table;
    static mut page_table: Table;
    //
    // static low_end: u8;
    // static kernel_size: u8; // &kernel_size == kernel size in pages
}

#[no_mangle]
pub unsafe fn start() {
    unimplemented!()
}

#[no_mangle]
#[naked]
unsafe fn create_page_tables() {
    const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024; // 2 MiB

    //-- map the PML4 and PDP tables -----------------------------------------
    // recursive map last PML4 entry
    pml4_table[511] = (&pml4_table as *const Table as u64) | 3;
    // map first PML4 entry to PDP table
    pml4_table[0] = (&pdp_table as *const Table as u64) | 3;
    // map first PDPT entry to PD table
    pdp_table[0] = (&pd_table as *const Table as u64) | 3;

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
}
