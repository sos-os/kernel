//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Initial 32-bit bootloader for x86_64
#![crate_name = "bootstrap"]
#![feature(asm)]
#![no_std]

const TABLE_LENGTH: usize = 512;

type Table = [u64; TABLE_LENGTH];

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
pub unsafe extern fn create_page_tables() {
    // recursive map last PML4 entry
    pml4_table[511] = (&pml4_table as *const Table as u64) | 3;
    // map first PML4 entry to PDP table
    pml4_table[0] = (&pdp_table as *const Table as u64) | 3;
    // map first PDPT entry to PD table
    pdp_table[0] = (&pd_table as *const Table as u64) | 3;

    // map the PD table
    // TODO: implement this in Rust
    asm!("
        // map each PD table entry to its own 2mB page
        mov         ecx, 0
    .pd_table_map: // maps the PD table ----------------------------------------
        mov     eax, 0x200000   // 2 mB
        mul     ecx             // times the start address of the page
        or      eax, 0b10000011 // check if present + writable + huge
        mov     [pd_table + ecx * 8], eax // map nth entry from pd -> own page
        // increment counter and check if done
        inc     ecx
        cmp     ecx, 512
        jne     .pd_table_map
        " :::: "volatile", "intel");
}
