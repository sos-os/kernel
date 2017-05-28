/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

ENTRY(_start)

SECTIONS {

    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .rodata :
    {
        /* This goes first. */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .boot : ALIGN(4K)
    {
        KEEP(*(.boot._start))
        libboot.a(*)
        KEEP(*(.gdt))
        . = ALIGN(4K);
    }

    .text :
    {
     /* NOTE we use KEEP here to prevent the linker from dropping
        these symbols
      */
        KEEP(*(.text.arch_init))
        *(.text .text.*)
        . = ALIGN(4K);
    }

     .data :
     {
       *(.data .data.*)
       . = ALIGN(4K);
     }

     .bss :
     {
         *(.bss .bss.*)
         . = ALIGN(4K);
        /* Page-Map Level-4 Table (PML4) */
        pml4_table = .;
        . += 4K;
        /* Page-Directory Pointer Table (PDP) */
        pdp_table = .;
        . += 4K;
        /* Page-Directory Table (PD) */
        pd_table = .;
        . += 4K;
            . = ALIGN(4K);
        stack_base = .;
        . += 4K * 8;
        stack_top = .;
            . = ALIGN(4K);
        heap_base_addr = .;
        . += 4K * 2K;
        heap_top_addr = .;
        . = ALIGN(4K);
     }

    .got :
    {
      *(.got)
      . = ALIGN(4K);
    }

    .got.plt :
    {
      *(.got.plt)
      . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K) {
      *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
      . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K) {
      *(.gcc_except_table)
      . = ALIGN(4K);
}
}
