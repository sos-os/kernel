# SOS x86_64 bootstrap

this crate contains code for the 32-bit protected mode boot routine for x86_64 CPUs. when we boot up an x86_64 system using GRUB & multiboot, we end up with the CPU running in 32-bit 'protected mode', rather than 64-bit 'long mode'. before we can jump into long mode, we have to perform a handful of setup tasks in protected mode.

this boot routine lives in a separate crate, with a 32-bit `target.json`. when building SOS for x86_64, we have to compile this crate separately and then link the resulting 32-bit object with the rest of the 64-bit kernel object.

this boot crate is only necessary for x86_64. for other architectures, all boot code can live in the main kernel crate.
