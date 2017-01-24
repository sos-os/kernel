/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

ENTRY(_start)

SECTIONS {
    . = 0xb8000;
    __vga_buffer = .;
    . += 80 * 24 * 2;

    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .boot :
    {
        /* This goes first. */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text :
    {
     /* NOTE we use KEEP here to prevent the linker from dropping
        these symbols
      */
        KEEP(*(.text.start))
        *(.text .text.*)
        . = ALIGN(4K);
    }

     .data :
     {
       *(.data .data.*)
       . = ALIGN(4K);
     }

     .bss ALIGN(4k) :
     {
        /* Page-Map Level-4 Table (PML4) */
        pml4_table = .;
        . += 4k;
        /* Page-Directory Pointer Table (PDP) */
        pdp_table = .;
        . += 4k;
        /* Page-Directory Table (PD) */
        pd_table = .;
        . += 4k;

        stack_base = .;
        . += 8k;
        stack_top = .;

        heap_base_addr = .;
        . += 4 * 1k * 1k;
        heap_top_addr = .;
     }

     .gdt :
     {
         KEEP(*(.gdt64))
         KEEP(*(gdt64_ptr))
        *(.rodata .rodata.*)
         . = ALIGN(4k);
     }

    .data.rel.ro : {
       *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K) {
      *(.gcc_except_table)
      . = ALIGN(4K);
    }
}
