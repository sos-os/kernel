//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use ::memory::VAddr;

/// A physical (linear) memory address is a 32-bit unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u32);

impl Addr<u32> for PAddr { }
impl_addr! { PAddr, u32 }

impl convert::Into<u32> for PAddr {
    #[inline] fn into(self) -> u32 { self.as_u32() }
}

impl convert::From<u32> for PAddr {
    #[inline] fn from(u: u32) -> Self { PAddr::from_u32(u) }
}

impl convert::From<*mut u8> for PAddr {
    #[inline] fn from(ptr: *mut u8) -> Self { PAddr::from_ptr(ptr) }
}

impl PAddr {
    #[inline] pub fn from_ptr(ptr: *mut u8) -> Self { PAddr(ptr as u32) }
    #[inline] pub const fn from_u32(u: u32) -> Self { PAddr(u) }
    #[inline] pub const fn as_u32(&self) -> u32 { self.0 }
}
