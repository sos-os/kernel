//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS memory allocation library
//!
//! This is in its own crate so it can be used by kernel-space and user-space
//! OS components.

#![crate_name = "sos_alloc"]
#![crate_type = "staticlib"]
#![feature(ptr_as_ref)]
#![feature(no_std)]
#![no_std]

mod rawlink;

#[test]
fn it_works() {
}
