#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]
#![feature(asm)]
#![feature(no_std, lang_items)]
#![feature(const_fn, unique, core_str_ext, core_slice_ext)]
#![no_std]

extern crate rlibc;
extern crate spin;

pub mod arch;
pub mod io;
use io::term::CONSOLE;

/// Kernel main loop
#[no_mangle]
pub extern fn kernel_main() {
    use core::fmt::Write;
    CONSOLE.lock()
           .clear()
           .write_str("Hello from the kernel!")
           .unwrap();
    loop { }
}

/// Required for Rust stack unwinding
#[lang = "eh_personality"]
extern fn eh_personality() {
    // TODO: add support for stack unwinding
}

#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {
    // TODO: actually format panics (waiting for robust VGA support)
    loop{}
}
