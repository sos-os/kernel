

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

        impl ::core::num::One for $ty {
            #[inline] fn one() -> Self {
                $ty { number:
                    <<$addr as Addr>::Repr as ::core::num::One>::one()
                }
            }
        }

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
