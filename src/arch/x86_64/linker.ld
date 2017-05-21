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

    .boot : ALIGN(4K) {
        /* This goes first. */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
    }

    .text : ALIGN(4K) {
     /* NOTE we use KEEP here to prevent the linker from dropping
        these symbols
      */
        KEEP(*(.text._start))
        KEEP(*(.text.arch_init))
        *(.text .text.*)
    }

     .data : ALIGN(4K)  {
       *(.data .data.*)
     }

     .bss : ALIGN(4K) {
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
        . += 4K * 2;
        heap_top_addr = .;
        *(.bss .bss.*)
     }

    .data.rel.ro : ALIGN(4K) {
       *(.data.rel.ro.local*)
       *(.data.rel.ro .data.rel.ro.*)
    }

    .gcc_except_table : ALIGN(4K) {
      *(.gcc_except_table)
      . = ALIGN(4096);
    }
}
