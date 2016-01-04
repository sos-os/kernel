//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Trait implementations for `PAddr`.

use core::fmt;
use core::ops;

use super::PAddr;

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
    ($(impl $name:ident, $fun:ident, $op:tt for PAddr)*) => {$(
        impl ops::$name<PAddr> for PAddr {
            type Output = PAddr;

            #[inline] fn $fun(self, _rhs: PAddr) -> PAddr {
                PAddr(e!(self.0 $op _rhs.0))
            }
        }
        impl ops::$name<u32> for PAddr {
            type Output = PAddr;

            #[inline] fn $fun(self, _rhs: u32) -> PAddr {
                PAddr(e!(self.0 $op _rhs))
            }
        }

        forward_ref_binop! {
            impl $name, $fun for PAddr, PAddr
        }
        forward_ref_binop! {
            impl $name, $fun for PAddr, u32
        }
    )*}
}
macro_rules! impl_fmt {
    ($(impl $name:ident for PAddr)*) => {$(
        impl fmt::$name for PAddr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    )*}
}


impl_ops! {
    impl Add, add, + for PAddr
    impl Sub, sub, - for PAddr
    impl Div, div, / for PAddr
    impl Mul, mul, * for PAddr
    impl Shl, shl, >> for PAddr
    impl Shr, shr, << for PAddr
}

impl_fmt! {
    impl Binary for PAddr
    impl Display for PAddr
    impl Octal for PAddr
    impl LowerHex for PAddr
    impl UpperHex for PAddr
}

impl ops::BitAnd<u32> for PAddr {
    type Output = u32;

    fn bitand(self, _rhs: u32) -> u32 {
        self.0 & _rhs
    }
}
