//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Panic handling and stack unwinding


use core::fmt;
use core::fmt::{Arguments, Write};
use super::io::term;

#[lang = "panic_fmt"]
#[no_mangle] #[inline(never)] #[cold]
pub extern fn panic_fmt( args: Arguments
                       , file: &'static str
                       , line: usize ) -> !
{
    write!( term::CONSOLE.lock()
          , "{}: {} {}"
          , file, line, args
          );
    loop { }
}

/// Required for Rust stack unwinding
#[lang = "eh_personality"]
#[no_mangle] #[inline(never)] #[cold]
pub extern fn eh_personality() {
    // TODO: add support for stack unwinding
}

#[lang = "stack_exhausted"]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn __morestack() -> ! {
    loop { }
}

#[allow(non_snake_case)]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop { }
}

#[lang = "begin_unwind"]
#[no_mangle] #[inline(never)] #[cold]
pub fn begin_unwind<M: Send>(msg: M
                            , file_line: &(&'static str, usize))  -> !
{ loop { } }
