/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

ENTRY(start)

SECTIONS {
    . = 0xb8000;
    vga_buffer = .;
    . += 80 * 24 * 2;

    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .boot :
    {
        /* This goes first. */
        KEEP(*(.multiboot))
    }

    .text :
    {
        *(.text .text.*)
    }

    .rodata : {
       *(.rodata .rodata.*)
    }

    .data.rel.ro : {
       *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }
}
