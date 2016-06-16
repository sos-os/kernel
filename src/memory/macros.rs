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

        impl Addr for $ty {
            type Repr = $size;
        }

        impl $crate::core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut $crate::core::fmt::Formatter)
                  -> $crate::core::fmt::Result {
                write!(f, "{}({:#x})", stringify!($ty), self.0)
            }
        }

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
            #[inline(always)]
            pub const fn as_mut_ptr<T>(&self) -> *mut T { self.0 as *mut _ }
            #[inline(always)]
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

    }
}


macro_rules! impl_page_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl $crate::core::ops::$name<$ty> for $ty {
            type Output = Self;

            #[inline] fn $fun(self, rhs: $ty) -> Self {
                expr!( $ty { number: expr!(self.number $op rhs.number) })
            }
        }
        impl $crate::core::ops::$name<$size> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $size) -> Self {
                expr!($ty{ number: expr!(self.number $op rhs) })
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

macro_rules! impl_page_assign_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl $crate::core::ops::$name<$ty> for $ty {

            #[inline] fn $fun(&mut self, rhs: $ty) {
                expr!(self.number $op rhs.number);
            }
        }
        impl $crate::core::ops::$name<$size> for $ty {

            #[inline] fn $fun(&mut self, rhs: $size) {
                expr!(self.number $op rhs);
            }
        }
    )*}
}



macro_rules! Page {
    (($addr:ty) $(pub)* struct $name:ident $($tail:tt)*) => {
        Page! { @impl $name, $addr }
    };
    (@impl $ty:ident, $addr:ty) => {

        impl Page for $ty {
            type Address = $addr;

            /// Create a new `Page` containing the given address.
            //  TODO: rewrite this as `up`/`down` using the page shift, instead.
            fn containing(addr: $addr) -> Self {
                use $crate::memory::PAGE_SHIFT;
                assert!( *addr < 0x0000_8000_0000_0000 ||
                         *addr >= 0xffff_8000_0000_0000
                       , "invalid address: 0x{:x}", addr );
                $ty { number: *addr >> PAGE_SHIFT as <Self::Address as Addr>::Repr }
            }

            /// Return the start virtual address of this page
            #[inline]
            fn base(&self) -> $addr {
                Self::Address::from(
                    self.number << PAGE_SHIFT as <Self::Address as Addr>::Repr )
            }

            #[inline] fn number(&self) -> usize {
                self.number as usize
            }
        }

        impl $crate::core::num::One for $ty {
            #[inline] fn one() -> Self {
                $ty { number:
                    <<$addr as Addr>::Repr as $crate::core::num::One>::one()
                }
            }
        }

        impl $crate::core::iter::Step for $ty {
            #[inline]
            fn step(&self, by: &Self) -> Option<Self> {
                self.number
                    .step(&by.number)
                    .map(|s| $ty { number: s})
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            fn steps_between(start: &$ty, end: &$ty, by: &$ty)
                            -> Option<usize> {
                use $crate::core::iter::Step;
                <<$addr as Addr>::Repr as Step>::steps_between( &start.number
                                                              , &end.number
                                                              , &by.number
                                                              )
            }
        }

        impl_page_ops! {
            Add, add, + for $ty, <<$ty as Page>::Address as Addr>::Repr
            Sub, sub, - for $ty, <<$ty as Page>::Address as Addr>::Repr
            Div, div, / for $ty, <<$ty as Page>::Address as Addr>::Repr
            Mul, mul, * for $ty, <<$ty as Page>::Address as Addr>::Repr
            Shl, shl, >> for $ty,<<$ty as Page>::Address as Addr>::Repr
            Shr, shr, << for $ty, <<$ty as Page>::Address as Addr>::Repr
            Rem, rem, % for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitAnd, bitand, & for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitOr, bitor, | for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitXor, bitxor, ^ for $ty, <<$ty as Page>::Address as Addr>::Repr

        }

        impl_page_assign_ops! {
            AddAssign, add_assign, += for $ty, <<$ty as Page>::Address as Addr>::Repr
            SubAssign, sub_assign, -= for $ty, <<$ty as Page>::Address as Addr>::Repr
            DivAssign, div_assign, /= for $ty, <<$ty as Page>::Address as Addr>::Repr
            MulAssign, mul_assign, *= for $ty, <<$ty as Page>::Address as Addr>::Repr
            ShlAssign, shl_assign, >>= for $ty, <<$ty as Page>::Address as Addr>::Repr
            ShrAssign, shr_assign, <<= for $ty, <<$ty as Page>::Address as Addr>::Repr
            RemAssign, rem_assign, %= for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitAndAssign, bitand_assign, &= for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitOrAssign, bitor_assign, |= for $ty, <<$ty as Page>::Address as Addr>::Repr
            BitXorAssign, bitxor_assign, ^= for $ty, <<$ty as Page>::Address as Addr>::Repr
        }
    }
}
