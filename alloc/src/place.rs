//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Support for placement expressions & box placement.
//!
//! See Rust [issue #27779] for more information.
//!
//! [issue #27779]: https://github.com/rust-lang/rust/issues/27779
use core::ptr::Unique;
use core::marker::PhantomData;

use super::{Layout, Allocator};

pub struct IntermediateAlloc<A, T>
where T: ?Sized
    , A: Allocator {
    ptr: Unique<u8>
  , marker: PhantomData<T>
  , alloc: A
}
