//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Macros to make our custom address types require a lot less repetitive code.
#[macro_export]
macro_rules! Addr {
    (($size:ty) $(pub)* enum $name:ident $($tail:tt)*) => {
        Addr! { @impl $name, $size }
    };
    (($size:ty) $(pub)* struct $name:ident $($tail:tt)*) => {
        Addr! { @impl $name, $size }
    };
    (@impl $ty:ident, $size:ty) => {

        impl ::core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                  -> ::core::fmt::Result {
                write!(f, "{}({:#x})", stringify!($ty), self.0)
            }
        }

        impl ::core::convert::Into<$size> for $ty {
            #[inline] fn into(self) -> $size { self.0 }
        }

        impl ::core::convert::From<$size> for $ty {
            #[inline] fn from(n: $size) -> Self { $ty(n) }
        }

        impl<T> ::core::convert::From<*mut T> for $ty {
            #[inline] fn from(ptr: *mut T) -> Self { $ty(ptr as $size) }
        }

        impl<T> ::core::convert::From<*const T> for $ty {
            #[inline] fn from(ptr: *const T) -> Self { $ty(ptr as $size) }
        }

        impl ::core::ops::Deref for $ty {
            type Target = $size;
            #[inline] fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl $ty {
            #[inline(always)]
            pub const fn as_mut_ptr<T>(&self) -> *mut T { self.0 as *mut _ }

            #[inline(always)]
            pub const fn as_ptr<T>(&self) -> *const T { self.0 as *const _ }

            /// Returns true if this address is aligned on a page boundary.
            #[inline]
            pub fn is_page_aligned(&self) -> bool {
                use $crate::arch::PAGE_SIZE;
                **self % PAGE_SIZE as <Self as Addr>::Repr == 0 as <Self as Addr>::Repr
            }
        }

        impl_ops! {
            Add, add, + for $ty, $size
            Sub, sub, - for $ty, $size
            Div, div, / for $ty, $size
            Mul, mul, * for $ty, $size
            Shl, shl, >> for $ty, $size
            Shr, shr, << for $ty, $size
            Rem, rem, % for $ty, $size
            BitAnd, bitand, & for $ty, $size
            BitOr, bitor, | for $ty, $size
            BitXor, bitxor, ^ for $ty, $size

        }

        impl_assign_ops! {
            AddAssign, add_assign, += for $ty, $size
            SubAssign, sub_assign, -= for $ty, $size
            DivAssign, div_assign, /= for $ty, $size
            MulAssign, mul_assign, *= for $ty, $size
            ShlAssign, shl_assign, >>= for $ty, $size
            ShrAssign, shr_assign, <<= for $ty, $size
            RemAssign, rem_assign, %= for $ty, $size
            BitAndAssign, bitand_assign, &= for $ty, $size
            BitOrAssign, bitor_assign, |= for $ty, $size
            BitXorAssign, bitxor_assign, ^= for $ty, $size
        }

        impl_fmt! {
            Binary for $ty
            Display for $ty
            Octal for $ty
            LowerHex for $ty
            UpperHex for $ty
        }

        impl Addr for $ty {
            type Repr = $size;
        }

    }
}
