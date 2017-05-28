//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
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

        impl ::core::fmt::Pointer for $ty {
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                  -> ::core::fmt::Result {
                write!(f, "{:#x}", self.0)
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

            #[inline(always)]
            pub const fn new(value: $size) -> Self { $ty(value) }


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

        impl ::core::cmp::PartialEq<$size> for $ty {
            #[inline] fn eq(&self, rhs: &$size) -> bool {
                self.0 == *rhs
            }
        }

        impl ::core::cmp::PartialOrd<$size> for $ty {
            #[inline]
            fn partial_cmp(&self, rhs: &$size) -> Option<::core::cmp::Ordering> {
                self.0.partial_cmp(rhs)
            }
        }

        impl Addr for $ty {
            type Repr = $size;

            #[inline] fn align_down(&self, align: Self::Repr) -> Self {
                use util::Align;
                $ty ( self.0.align_down(align) )
            }

            #[inline] fn align_up(&self, align: Self::Repr) -> Self {
                use util::Align;
                // assert!(align.is_page_aligned());
                $ty ( self.0.align_up(align) )
            }

            #[inline] fn is_page_aligned(&self) -> bool {
                **self % PAGE_SIZE as <Self as Addr>::Repr == 0 as <Self as Addr>::Repr
            }
        }

    }
}



macro_rules! impl_page_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl ::core::ops::$name<$ty> for $ty {
            type Output = Self;

            #[inline] fn $fun(self, rhs: $ty) -> Self {
                expr!( $ty { number: expr!(self.number $op rhs.number) })
            }
        }
        impl ::core::ops::$name<$size> for $ty {
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
        impl ::core::ops::$name<$ty> for $ty {

            #[inline] fn $fun(&mut self, rhs: $ty) {
                expr!(self.number $op rhs.number);
            }
        }
        impl ::core::ops::$name<$size> for $ty {

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
                use ::PAGE_SHIFT;
                assert!( *addr < 0x0000_8000_0000_0000 ||
                         *addr >= 0xffff_8000_0000_0000
                       , "invalid address: 0x{:x}", addr );
                $ty { number: *addr >> PAGE_SHIFT as <Self::Address as Addr>::Repr }
            }

            /// Return the start address of this page
            #[inline]
            fn base(&self) -> $addr {
                Self::Address::from(
                    self.number << PAGE_SHIFT as <Self::Address as Addr>::Repr )
            }

            /// Return the end address of this page
            #[inline]
            fn end_address(&self) -> $addr {
                self.base() + PAGE_SIZE as <Self::Address as Addr>::Repr
            }

            #[inline] fn number(&self) -> usize {
                self.number as usize
            }
        }

        // impl ::core::num::One for $ty {
        //     #[inline] fn one() -> Self {
        //         $ty { number:
        //             <<$addr as Addr>::Repr as ::core::num::One>::one()
        //         }
        //     }
        // }

        impl ::core::iter::Step for $ty {
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
                use ::core::iter::Step;
                <<$addr as Addr>::Repr as Step>::steps_between( &start.number
                                                              , &end.number
                                                              , &by.number
                                                              )
            }

            #[inline]
            fn steps_between_by_one(start: &$ty, end: &$ty) -> Option<usize> {
                use ::core::iter::Step;
                <<$addr as Addr>::Repr as Step>::steps_between( &start.number
                                                              , &end.number
                                                              , &1
                                                              )
            }

            #[inline] fn sub_one(&self) -> Self { self - 1 }

            #[inline] fn add_one(&self) -> Self { self + 1 }

            #[inline] fn replace_one(&mut self) -> Self { unimplemented!() }

            #[inline] fn replace_zero(&mut self) -> Self { unimplemented!() }

            #[inline] fn is_negative(&self) -> bool { unimplemented!(); }

        }

        impl<A> ::core::convert::From<A> for $ty
        where <Self as Page>::Address: ::core::convert::From<A> {
            #[inline] fn from(addr: A) -> Self {
                $ty::containing(<Self as Page>::Address::from(addr))
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
