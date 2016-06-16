
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

macro_rules! impl_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl $crate::core::ops::$name<$ty> for $ty {
            type Output = $ty;

            #[inline] fn $fun(self, rhs: $ty) -> $ty {
                $ty(expr!(self.0 $op rhs.0))
            }
        }
        impl $crate::core::ops::$name<$size> for $ty {
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

macro_rules! impl_assign_ops {
    ($($name:ident, $fun:ident, $op:tt for $ty:ident, $size:ty)*) => {$(
        impl $crate::core::ops::$name<$ty> for $ty {

            #[inline] fn $fun(&mut self, rhs: $ty) {
                expr!(self.0 $op rhs.0);
            }
        }
        impl $crate::core::ops::$name<$size> for $ty {

            #[inline] fn $fun(&mut self, rhs: $size) {
                expr!(self.0 $op rhs);
            }
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
