/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

ENTRY(start)

SECTIONS {
    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .boot :
    {
        /* This goes first. */
        KEEP(*(.multiboot))
    }

    .text :
    {
        *(.text)
    }
}
