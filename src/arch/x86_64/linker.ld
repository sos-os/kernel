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
        . = ALIGN(4096);
    }

    .text :
    {
     /* NOTE we use KEEP here to prevent the linker from dropping
        these symbols
      */
        KEEP(*(.text._start))
        KEEP(*(.text.arch_init))
        *(.text .text.*)
        . = ALIGN(4096);
    }

    .gdt :
    {
        KEEP(*(.gdt64))
        KEEP(*(gdt64_ptr))
       *(.rodata .rodata.*)
        . = ALIGN(4096);
    }

     .data :
     {
       *(.data .data.*)
       . = ALIGN(4096);
     }

     .bss :
     {
        /* Page-Map Level-4 Table (PML4) */
        pml4_table = .;
        . += 4096;
        /* Page-Directory Pointer Table (PDP) */
        pdp_table = .;
        . += 4096;
        /* Page-Directory Table (PD) */
        pd_table = .;
        . += 4096;


        stack_base = .;
        . += 4K * 2;
        stack_top = .;

        heap_base_addr = .;
        . += 4 * 1k * 1k;
        heap_top_addr = .;

        . = ALIGN(4096);
     }

    .got :
    {
      *(.got)
      . = ALIGN(4096);
    }

    .got.plt :
    {
      *(.got.plt)
      . = ALIGN(4096);
    }

    .data.rel.ro : {
       *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4096);
    }

    .gcc_except_table : ALIGN(4096) {
      *(.gcc_except_table)
      . = ALIGN(4096);
    }
}
