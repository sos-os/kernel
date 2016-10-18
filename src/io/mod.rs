//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Kernel IO.
//!
//! This module should eventually abstract over architecture-specific
//! implementation.
pub mod term;
pub mod keyboard;

// use arch::cpu;

use core::{ ops, fmt };
use core::marker::PhantomData;

// macro_rules! println {
//     ($fmt:expr) => (print!(concat!($fmt, "\n")));
//     ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
// }
//
// macro_rules! print {
//     ($($arg:tt)*) => ({
//             use core::fmt::Write;
//             $crate::io::term::CONSOLE.lock()
//                                      .write_fmt(format_args!($($arg)*))
//                                      .unwrap();
//     });
// }
