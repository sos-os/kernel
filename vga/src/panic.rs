//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Panic handling.
//!
//! This module contains the `panic_fmt` language item. This function handles
//! panics at runtime.

use core::fmt::{Arguments, Write};
use super::{Color, CONSOLE};

/// Called to handle a panic.
///
/// Since kernel panics are non-recoverable, this function prints out
/// the error message and hangs forever.
///
/// Eventually – way in the future – when we have disk I/O and stuff,
/// we'll probably want to write out some core dumps here as well.
#[lang = "panic_fmt"]
#[no_mangle] #[inline(never)] #[cold]
pub extern "C" fn rust_begin_unwind( args: Arguments
                                   , file: &'static str
                                   , line: usize )
                                   -> ! {
    let _ = write!( CONSOLE.lock()
                        .set_colors(Color::White, Color::Red)
                  , "Something has gone horribly wrong in {} at line {}. \
                    \n{}\n\
                    This is fine."
                  , file, line, args
                  );
    loop { }
}
