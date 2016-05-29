/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

ENTRY(start)

SECTIONS {
    . = 0xb8000;
    __vga_buffer = .;
    . += 80 * 24 * 2;

    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .boot :
    {
        /* This goes first. */
        KEEP(*(.multiboot))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }
    .text :
     {
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
     }

    .rodata : {
       *(.rodata .rodata.*)
        . = ALIGN(4K);
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
