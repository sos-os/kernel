//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Macros to make our custom address types require a lot less repetitive code.

macro_rules! Addr {
    (($size:ty) $(pub)* enum $name:ident $($tail:tt)*) => {
        Addr! { @impl $name, $size }
    };
    (($size:ty) $(pub)* struct $name:ident $($tail:tt)*) => {
        Addr! { @impl $name, $size }
    };
    (@impl $ty:ident, $size:ty) => {

        impl Addr<$size> for $ty { }

        impl $crate::core::convert::Into<$size> for $ty {
            #[inline] fn into(self) -> $size { self.0 }
        }

        impl $crate::core::convert::From<$size> for $ty {
            #[inline] fn from(n: $size) -> Self { $ty(n) }
        }

        impl<T> $crate::core::convert::From<*mut T> for $ty {
            #[inline] fn from(ptr: *mut T) -> Self { $ty(ptr as $size) }
        }

        impl<T> $crate::core::convert::From<*const T> for $ty {
            #[inline] fn from(ptr: *const T) -> Self { $ty(ptr as $size) }
        }

        impl $crate::core::ops::Deref for $ty {
            type Target = $size;
            #[inline] fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl $ty {
            pub const fn as_mut_ptr<T>(&self) -> *mut T { self.0 as *mut _ }
            pub const fn as_ptr<T>(&self) -> *const T { self.0 as *const _ }
        }

        impl_ops! {
            Add, add, + for $ty, $size
            Sub, sub, - for $ty, $size
            Div, div, / for $ty, $size
            Mul, mul, * for $ty, $size
            Shl, shl, >> for $ty, $size
            Shr, shr, << for $ty, $size
            Rem, rem, % for $ty, $size
        }

        impl_fmt! {
            Binary for $ty
            Display for $ty
            Octal for $ty
            LowerHex for $ty
            UpperHex for $ty
        }

        impl $crate::core::ops::BitAnd<$size> for $ty {
            type Output = $size;

            fn bitand(self, rhs: $size) -> $size {
                self.0 & rhs
            }
        }
    }
}

macro_rules! forward_ref_binop {
    ($imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $crate::core::ops::$imp<$u> for &'a $t {
            type Output = <$t as $crate::core::ops::$imp<$u>>::Output;
            #[inline]
            fn $method(self, other: $u)
                      -> <$t as $crate::core::ops::$imp<$u>>::Output {
                $crate::core::ops::$imp::$method(*self, other)
            }
        }

        impl<'a> $crate::core::ops::$imp<&'a $u> for $t {
            type Output = <$t as $crate::core::ops::$imp<$u>>::Output;
            #[inline]
            fn $method(self, other: &'a $u)
                      -> <$t as $crate::core::ops::$imp<$u>>::Output {
                $crate::core::ops::$imp::$method(self, *other)
            }
        }

        impl<'a, 'b> $crate::core::ops::$imp<&'a $u> for &'b $t {
            type Output = <$t as $crate::core::ops::$imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &'a $u) -> <$t as $crate::core::ops::$imp<$u>>::Output {
                $crate::core::ops::$imp::$method(*self, *other)
            }
        }
    }
}
macro_rules! e { ($e:expr) => { $e } }

macro_rules! impl_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl $crate::core::ops::$name<$ty> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $ty) -> $ty {
                $ty(e!(self.0 $op rhs.0))
            }
        }
        impl $crate::core::ops::$name<$size> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $size) -> $ty {
                $ty(e!(self.0 $op rhs))
            }
        }

        forward_ref_binop! {
            $name, $fun for $ty, $ty
        }
        forward_ref_binop! {
            $name, $fun for $ty, $size
        }
    )*}
}

macro_rules! impl_fmt {
    ($($name:ident for $ty:ty)*) => {$(
        impl $crate::core::fmt::$name for $ty {
            fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
                self.0.fmt(f)
            }
        }
    )*}
}
