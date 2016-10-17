#![allow(missing_docs)]

macro_rules! expr { ($e:expr) => { $e } }

#[macro_use] pub mod newtype_impl;
#[macro_use] pub mod log;
