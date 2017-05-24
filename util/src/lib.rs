//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! General purpose stuff I couldn't find a better home for.
#![crate_name = "util"]
#![no_std]

// #[cfg(not(test))] extern crate vga;

use core::fmt;

pub mod io;

#[macro_use] pub mod macros;

/// The unreachable Void type.
pub enum Void {}
impl fmt::Debug for Void {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}
