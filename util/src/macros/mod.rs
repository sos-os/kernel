//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

#![allow(missing_docs)]
#[macro_export]
macro_rules! expr { ($e:expr) => { $e } }

#[macro_use] pub mod newtype_impl;
// #[macro_use] pub mod log;
