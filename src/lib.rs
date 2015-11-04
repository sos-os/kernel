#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]
#![feature(asm)]
#![feature(no_std, lang_items)]
#![feature(const_fn, unique)]
#![no_std]

extern crate rlibc;

pub mod arch;

/// Kernel main loop
#[no_mangle]
pub extern fn kernel_main() {}

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
