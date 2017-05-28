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
use core::ops::{BoxPlace, Place, Placer, InPlace};
use core::ptr::Unique;
use core::marker::PhantomData;

use super::{Layout, Allocator};

use spin::Mutex;

/// A [`Place`] representing an intermediate allocoation attempt.
///
/// [`Place`]: https://doc.rust-lang.org/std/ops/trait.Place.html
pub struct IntermediateAlloc<'alloc, A, T>
where T: ?Sized
    , A: Allocator {
    ptr: Unique<T>
  , layout: Layout
  , alloc: &'alloc Mutex<A>
}

impl Placer<'alloc, Data, A> for &'alloc Mutex<A>
where A: Allocator
    , Data: ? Sized {

        type Place = IntermediateAlloc<'alloc, A, Data>;

        /// Creates a fresh place from `self`.
        fn make_place(self) -> Self::Place {
            IntermediateAlloc { ptr: self.lock()
                                         .alloc_one::<T>()
                                         .unwrap()
                              , layout: Layout::new::<T>()
                              , alloc: self }
        }
}

impl<'alloc, A, Data> Place<Data> for IntermediateAlloc<'alloc, A, Data>
where Data: ?Sized
    , A: Allocator {

        #[inline] fn pointer(&mut self) -> *mut Data {
            self.ptr
        }
}

impl<'alloc, A, Data> InPlace<Data> for IntermediateAlloc<'alloc, A, Data>
where Data: ?Sized
    , A: Allocator {

        /// `Owner` is the type of the end value of `in (PLACE) EXPR`
        ///
        /// Note that when `in (PLACE) EXPR` is solely used for
        /// side-effecting an existing data-structure,
        /// e.g. `Vec::emplace_back`, then `Owner` need not carry any
        /// information at all (e.g. it can be the unit type `()` in that
        /// case).
        type Owner = Data;

        /// Converts self into the final value, shifting
        /// deallocation/cleanup responsibilities (if any remain), over to
        /// the returned instance of `Owner` and forgetting self.
        unsafe fn finalize(self) -> Self::Owner {
            unimplemented!()
        }
}
