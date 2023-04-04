use argus_core::expr::BoolExpr;
use argus_core::prelude::*;

use crate::eval::NumExprEval;
use crate::{Semantics, Trace};

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

/// Boolean semantics of Argus expressions
pub struct BooleanSemantics;

impl Semantics for BooleanSemantics {
    // TODO: figure out how to make Output concrete Signal<bool> or ConstantSignal<bool>
    type Output = AnySignal;
    type Context = ();

    fn eval(expr: &BoolExpr, trace: &impl Trace, ctx: Self::Context) -> ArgusResult<Self::Output> {
        match expr {
            BoolExpr::BoolLit(val) => Ok(ConstantSignal::new(*val).into()),
            BoolExpr::BoolVar { name } => trace.get(name.as_str()).cloned().ok_or(ArgusError::SignalNotPresent),
            BoolExpr::Cmp { op, lhs, rhs } => {
                use argus_core::expr::Ordering::*;
                let lhs = NumExprEval::eval(lhs, trace);
                let rhs = NumExprEval::eval(rhs, trace);
                let ret = match op {
                    Eq => signal_cmp_op_impl!(lhs == rhs),
                    NotEq => signal_cmp_op_impl!(lhs != rhs),
                    Less { strict } if *strict => signal_cmp_op_impl!(lhs < rhs),
                    Less { strict: _ } => signal_cmp_op_impl!(lhs <= rhs),
                    Greater { strict } if *strict => signal_cmp_op_impl!(lhs > rhs),
                    Greater { strict: _ } => signal_cmp_op_impl!(lhs >= rhs),
                };
                ret.ok_or(ArgusError::InvalidOperation)
            }
            BoolExpr::Not { arg } => {
                let arg = Self::eval(arg, trace, ctx)?;
                Ok(signal_bool_op_impl!(!arg))
            }
            BoolExpr::And { args } => {
                let args: ArgusResult<Vec<_>> = args.iter().map(|arg| Self::eval(arg, trace, ctx)).collect();
                let ret = args?
                    .into_iter()
                    .fold(AnySignal::from(ConstantSignal::new(true)), |lhs, rhs| {
                        signal_bool_op_impl!(lhs & rhs)
                    });
                Ok(ret)
            }
            BoolExpr::Or { args } => {
                let args: ArgusResult<Vec<_>> = args.iter().map(|arg| Self::eval(arg, trace, ctx)).collect();
                let ret = args?
                    .into_iter()
                    .fold(AnySignal::from(ConstantSignal::new(true)), |lhs, rhs| {
                        signal_bool_op_impl!(lhs | rhs)
                    });
                Ok(ret)
            }
        }
    }
}
