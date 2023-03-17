macro_rules! forward_box_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl $imp<$u> for Box<$t> {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<Box<$u>> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: Box<$u>) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<Box<$u>> for Box<$t> {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: Box<$u>) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

pub(crate) use forward_box_binop;
