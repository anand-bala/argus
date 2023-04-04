use argus_core::expr::NumExpr;
use argus_core::signals::{AnySignal, ConstantSignal};

use crate::Trace;

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

pub(crate) use signal_num_op_impl;

/// Helper struct to evaluate a [`NumExpr`] given a trace.
pub struct NumExprEval;

impl NumExprEval {
    pub fn eval(root: &NumExpr, trace: &impl Trace) -> AnySignal {
        match root {
            NumExpr::IntLit(val) => ConstantSignal::new(*val).into(),
            NumExpr::UIntLit(val) => ConstantSignal::new(*val).into(),
            NumExpr::FloatLit(val) => ConstantSignal::new(*val).into(),
            NumExpr::IntVar { name } | NumExpr::UIntVar { name } | NumExpr::FloatVar { name } => {
                // TODO(anand): Type check!
                trace.get(name.as_str()).cloned().unwrap()
            }
            NumExpr::Neg { arg } => {
                let arg_sig = Self::eval(arg, trace);
                signal_num_op_impl!(-arg_sig)
            }
            NumExpr::Add { args } => {
                let args_signals = args.iter().map(|arg| Self::eval(arg, trace));
                args_signals.reduce(|acc, arg| signal_num_op_impl!(acc + arg)).unwrap()
            }
            NumExpr::Mul { args } => {
                let args_signals = args.iter().map(|arg| Self::eval(arg, trace));
                args_signals.reduce(|acc, arg| signal_num_op_impl!(acc * arg)).unwrap()
            }
            NumExpr::Div { dividend, divisor } => {
                let dividend = Self::eval(dividend, trace);
                let divisor = Self::eval(divisor, trace);
                signal_num_op_impl!(dividend / divisor)
            }
        }
    }
}
