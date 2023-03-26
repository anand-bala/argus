macro_rules! signal_num_op_impl {
    // Unary numeric opeartions
    (- $signal:ident) => {{
        use argus_core::prelude::*;
        use AnySignal::*;
        match $signal {
            Bool(_) | ConstBool(_) => panic!("cannot perform unary operation (-) on Boolean signals"),
            Int(signal) => AnySignal::from(-(&signal)),
            ConstInt(signal) => AnySignal::from(-(&signal)),
            UInt(_) | ConstUInt(_) => panic!("cannot perform unary operation (-) on unsigned integer signals"),
            Float(signal) => AnySignal::from(-(&signal)),
            ConstFloat(signal) => AnySignal::from(-(&signal)),
        }
    }};

    ($lhs:ident $op:tt $rhs:ident, [$( $type:ident ),*]) => {
        paste::paste!{
            {
            use argus_core::prelude::*;
            use AnySignal::*;
            match ($lhs, $rhs) {
                (Bool(_), _) | (ConstBool(_), _) | (_, Bool(_)) | (_, ConstBool(_)) => panic!("cannot perform numeric operation {} for boolean arguments", stringify!($op)),
                $(
                    ([<$type >](lhs), [<  $type >](rhs)) => AnySignal::from(&lhs $op &rhs),
                    ([<$type >](lhs), [< Const $type >](rhs)) => AnySignal::from(&lhs $op &rhs),
                    ([<Const $type >](lhs), [<  $type >](rhs)) => AnySignal::from(&lhs $op &rhs),
                    ([<Const $type >](lhs), [< Const $type >](rhs)) => AnySignal::from(&lhs $op &rhs),
                )*
                _ => panic!("mismatched argument types for {} operation", stringify!($op)),
                }
            }
        }
    };

    // Binary numeric opeartions
    ($lhs:ident $op:tt $rhs:ident) => {
        signal_num_op_impl!(
            $lhs $op $rhs,
            [Int, UInt, Float]
        )
    };
}

macro_rules! signal_cmp_op_impl {
    ($lhs:ident, $rhs:ident, $op:ident, [$( $type:ident ),*]) => {
        paste::paste!{
            {
            use argus_core::signals::traits::SignalPartialOrd;
            use argus_core::prelude::*;
            use AnySignal::*;
            match ($lhs, $rhs) {
                (Bool(_), _) | (ConstBool(_), _) | (_, Bool(_)) | (_, ConstBool(_)) => panic!("cannot perform comparison operation ({}) for boolean arguments", stringify!($op)),
                $(
                    ([<$type >](lhs), [<  $type >](rhs)) => lhs.$op(&rhs).map(AnySignal::from),
                    ([<$type >](lhs), [< Const $type >](rhs)) => lhs.$op(&rhs).map(AnySignal::from),
                    ([<Const $type >](lhs), [<  $type >](rhs)) => lhs.$op(&rhs).map(AnySignal::from),
                    ([<Const $type >](lhs), [< Const $type >](rhs)) => lhs.$op(&rhs).map(AnySignal::from),
                )*
                _ => panic!("mismatched argument types for comparison operation ({})", stringify!($op)),
                }
            }
        }
    };

    ($lhs:ident < $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_lt, [Int, UInt, Float])
    };

    ($lhs:ident <= $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_le, [Int, UInt, Float])
    };

    ($lhs:ident > $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_gt, [Int, UInt, Float])
    };
    ($lhs:ident >= $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_ge, [Int, UInt, Float])
    };

    ($lhs:ident == $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_eq, [Int, UInt, Float])
    };

    ($lhs:ident != $rhs:ident) => {
        signal_cmp_op_impl!($lhs, $rhs, signal_ne, [Int, UInt, Float])
    };
}

macro_rules! signal_bool_op_impl {
    // Unary bool opeartions
    (! $signal:ident) => {{
        use argus_core::prelude::*;
        use AnySignal::*;
        match $signal {
            Bool(sig) => AnySignal::from(!(&sig)),
            ConstBool(sig) => AnySignal::from(!(&sig)),
            _ => panic!("cannot perform unary operation (!) on numeric signals"),
        }
    }};

    ($lhs:ident $op:tt $rhs:ident) => {
        paste::paste! {
            {
            use argus_core::prelude::*;
            use AnySignal::*;
            match ($lhs, $rhs) {
                (Bool(lhs), Bool(rhs)) => AnySignal::from(&lhs $op &rhs),
                (Bool(lhs), ConstBool(rhs)) => AnySignal::from(&lhs $op &rhs),
                (ConstBool(lhs), Bool(rhs)) => AnySignal::from(&lhs $op &rhs),
                (ConstBool(lhs), ConstBool(rhs)) => AnySignal::from(&lhs $op &rhs),
                _ => panic!("mismatched argument types for {} operation", stringify!($op)),
                }
            }
        }
    };
}

pub(crate) use {signal_bool_op_impl, signal_cmp_op_impl, signal_num_op_impl};
