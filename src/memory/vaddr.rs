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
        (self >> 39) & 0b111111111
    }

    /// Calculate the index in the PDPT table corresponding to this address.
    #[inline] pub fn pdpt_index(&self) -> usize {
        (self >> 30) & 0b111111111
    }

    /// Calculate the index in the PD table corresponding to this address.
    #[inline] pub fn pd_index(&self) -> usize {
        (self >> 21) & 0b111111111
    }

    /// Calculate the index in the PT table corresponding to this address.
    #[inline] pub fn pt_index(&self) -> usize {
        (self >> 12) & 0b111111111
    }
}

macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> ops::$imp<$u> for &'a $t {
            type Output = <$t as ops::$imp<$u>>::Output;
            #[inline]
            fn $method(self, other: $u) -> <$t as ops::$imp<$u>>::Output {
                ops::$imp::$method(*self, other)
            }
        }

        impl<'a> ops::$imp<&'a $u> for $t {
            type Output = <$t as ops::$imp<$u>>::Output;
            #[inline]
            fn $method(self, other: &'a $u) -> <$t as ops::$imp<$u>>::Output {
                ops::$imp::$method(self, *other)
            }
        }

        impl<'a, 'b> ops::$imp<&'a $u> for &'b $t {
            type Output = <$t as ops::$imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &'a $u) -> <$t as ops::$imp<$u>>::Output {
                ops::$imp::$method(*self, *other)
            }
        }
    }
}
macro_rules! e { ($e:expr) => { $e } }
macro_rules! impl_ops {
    ($(impl $name:ident, $fun:ident, $op:tt for VAddr)*) => {$(
        impl ops::$name<VAddr> for VAddr {
            type Output = VAddr;

            #[inline] fn $fun(self, _rhs: VAddr) -> VAddr {
                VAddr(e!(self.0 $op _rhs.0))
            }
        }
        impl ops::$name<usize> for VAddr {
            type Output = VAddr;

            #[inline] fn $fun(self, _rhs: usize) -> VAddr {
                VAddr(e!(self.0 $op _rhs))
            }
        }

        forward_ref_binop! {
            impl $name, $fun for VAddr, VAddr
        }
        forward_ref_binop! {
            impl $name, $fun for VAddr, usize
        }
    )*}
}
macro_rules! impl_fmt {
    ($(impl $name:ident for VAddr)*) => {$(
        impl fmt::$name for VAddr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    )*}
}


impl_ops! {
    impl Add, add, + for VAddr
    impl Sub, sub, - for VAddr
    impl Div, div, / for VAddr
    impl Mul, mul, * for VAddr
    impl Shl, shl, >> for VAddr
    impl Shr, shr, << for VAddr
}

impl_fmt! {
    impl Binary for VAddr
    impl Display for VAddr
    impl Octal for VAddr
    impl LowerHex for VAddr
    impl UpperHex for VAddr
}

impl ops::BitAnd<usize> for VAddr {
    type Output = usize;

    fn bitand(self, _rhs: usize) -> usize {
        self.0 & _rhs
    }
}
