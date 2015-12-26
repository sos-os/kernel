//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Panic handling and stack unwinding

use core::fmt::{Arguments, Write};
use super::io::term;
use vga::Color;

#[lang = "panic_fmt"]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn rust_begin_unwind( args: Arguments, file: &'static str
                               , line: usize )
                               -> ! {
    write!(term::CONSOLE.lock()
                        .set_colors(Color::White, Color::Red)
                        .clear()
          , "KERNEL PANIC in {} at line {}\
            \nSomething has gone horribly wrong: {}. \
            This is fine."
          , file, line, args
          );
    loop { }
}

#[lang = "stack_exhausted"]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn __morestack() -> ! {
    println!("stack exhausted");
    loop { }
}

#[allow(non_snake_case)]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn _Unwind_Resume() -> ! {
    println!("UNWIND!");
    loop { }
}


// #[lang = "begin_unwind"]
// #[no_mangle] #[inline(never)] #[cold]
// pub fn begin_unwind<M: Send>(msg: M
//                             , file_line: &(&'static str, usize))  -> !
// { loop { } }
