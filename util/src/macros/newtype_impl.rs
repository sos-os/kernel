//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

#[macro_export]
macro_rules! forward_ref_binop {
    ($imp:ident, $method:ident for $t:ty, $u:ty) => {
            impl<'a> core::ops::$imp<$u> for &'a $t {
                type Output = <$t as core::ops::$imp<$u>>::Output;
                #[inline]
                fn $method(self, other: $u)
                          -> <$t as core::ops::$imp<$u>>::Output {
                    core::ops::$imp::$method(*self, other)
                }
            }

            impl<'a> core::ops::$imp<&'a $u> for $t {
                type Output = <$t as core::ops::$imp<$u>>::Output;
                #[inline]
                fn $method(self, other: &'a $u)
                          -> <$t as core::ops::$imp<$u>>::Output {
                    core::ops::$imp::$method(self, *other)
                }
            }

            impl<'a, 'b> core::ops::$imp<&'a $u> for &'b $t {
                type Output = <$t as core::ops::$imp<$u>>::Output;

                #[inline]
                fn $method(self, other: &'a $u) -> <$t as core::ops::$imp<$u>>::Output {
                    core::ops::$imp::$method(*self, *other)
                }
            }
    };
}
#[macro_export]
macro_rules! impl_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl core::ops::$name<$ty> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $ty) -> $ty {
                $ty(expr!(self.0 $op rhs.0))
            }
        }
        impl core::ops::$name<$size> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $size) -> $ty {
                $ty(expr!(self.0 $op rhs))
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
#[macro_export]
macro_rules! impl_assign_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl core::ops::$name<$ty> for $ty {

            #[inline] fn $fun(&mut self, rhs: $ty) {
                expr!(self.0 $op rhs.0);
            }
        }
        impl core::ops::$name<$size> for $ty {

            #[inline] fn $fun(&mut self, rhs: $size) {
                expr!(self.0 $op rhs);
            }
        }
    )*}
}
#[macro_export]
macro_rules! impl_fmt {
    ($($name:ident for $ty:ty)*) => {$(
        impl core::fmt::$name for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }
    )*}
}
