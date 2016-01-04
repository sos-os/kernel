//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Trait implementations for `VAddr`

use core::fmt;
use core::ops;

use super::VAddr;

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
