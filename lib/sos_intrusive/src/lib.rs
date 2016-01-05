//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS intrusive data structures library
//!
//! These structures are primarily used by the kernel and memory allocator.
#![crate_name = "sos_ds"]
#![crate_type = "lib"]
#![feature(const_fn, ptr_as_ref, unique)]
#![feature(no_std)]
#![no_std]

mod rawlink;
// use rawlink::RawLink;
pub mod list;
// pub use list::List;

#[cfg(test)]
extern crate std;
