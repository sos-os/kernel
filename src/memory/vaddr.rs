//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Type representing virtual addresses.

use core::fmt;
use core::ops;

/// A virtual address is a machine-sized unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(usize);

impl VAddr {
    #[inline] pub const fn from_usize(u: usize) -> Self {
        VAddr(u)
    }
    #[inline] pub const fn as_usize(&self) -> usize {
        self.0
    }

    /// Calculate the index in the PML4 table corresponding to this address.
    #[inline] pub fn pml4_index(&self) -> usize {
        (*self >> 39) & 0b111111111
    }

    /// Calculate the index in the PDPT table corresponding to this address.
    #[inline] pub fn pdpt_index(&self) -> usize {
        (*self >> 30) & 0b111111111
    }

    /// Calculate the index in the PD table corresponding to this address.
    #[inline] pub fn pd_index(&self) -> usize {
        (*self >> 21) & 0b111111111
    }

    /// Calculate the index in the PT table corresponding to this address.
    #[inline] pub fn pt_index(&self) -> usize {
        (*self >> 12) & 0b111111111
    }
}

impl fmt::Binary for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ops::Add<VAddr> for VAddr {
    type Output = VAddr;

    fn add(self, _rhs: VAddr) -> VAddr {
        VAddr(self.0 + _rhs.0)
    }
}

impl ops::Sub<VAddr> for VAddr {
    type Output = VAddr;

    fn sub(self, _rhs: VAddr) -> VAddr {
        VAddr(self.0 - _rhs.0)
    }
}

impl ops::Div<VAddr> for VAddr {
    type Output = VAddr;

    fn div(self, _rhs: VAddr) -> VAddr {
        VAddr(self.0 / _rhs.0)
    }
}

impl ops::Mul<VAddr> for VAddr {
    type Output = VAddr;

    fn mul(self, _rhs: VAddr) -> VAddr {
        VAddr(self.0 * _rhs.0)
    }
}

impl ops::Add<usize> for VAddr {
    type Output = VAddr;

    fn add(self, _rhs: usize) -> VAddr {
        VAddr(self.0 + _rhs)
    }
}

impl ops::Sub<usize> for VAddr {
    type Output = VAddr;

    fn sub(self, _rhs: usize) -> VAddr {
        VAddr(self.0 - _rhs)
    }
}

impl ops::Div<usize> for VAddr {
    type Output = VAddr;

    fn div(self, _rhs: usize) -> VAddr {
        VAddr(self.0 / _rhs)
    }
}

impl ops::Mul<usize> for VAddr {
    type Output = VAddr;

    fn mul(self, _rhs: usize) -> VAddr {
        VAddr(self.0 * _rhs)
    }
}

impl ops::Shl<usize> for VAddr {
    type Output = VAddr;

    fn shl(self, _rhs: usize) -> VAddr {
        VAddr(self.0 << _rhs)
    }
}

impl ops::Shr<usize> for VAddr {
    type Output = VAddr;

    fn shr(self, _rhs: usize) -> VAddr {
        VAddr(self.0 >> _rhs)
    }
}

impl ops::BitAnd<usize> for VAddr {
    type Output = usize;

    fn bitand(self, _rhs: usize) -> usize {
        self.0 & _rhs
    }
}
