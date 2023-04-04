use argus_core::expr::NumExpr;
use argus_core::signals::{AnySignal, ConstantSignal};

use crate::Trace;

macro_rules! signal_num_op_impl {
    // Unary numeric opeartions
    ($op:ident, $signal:ident, [$( $type:ident ),*]) => {
        paste::paste! {
            {
                use argus_core::prelude::*;
                use AnySignal::*;
                match $signal {
                    $(
                        [< $type >](signal) => AnySignal::from(signal.$op()),
                        [<Const $type >](signal) => AnySignal::from(signal.$op()),
                    )*
                    _ => panic!("cannot perform unary operation ({})", stringify!($op)),
                }
            }
        }
    };

    ($op:ident, $lhs:ident, $rhs:ident, [$( $type:ident ),*]) => {
        paste::paste!{
            {
            use argus_core::prelude::*;
            use AnySignal::*;
            match ($lhs, $rhs) {
                (Bool(_), _) | (ConstBool(_), _) | (_, Bool(_)) | (_, ConstBool(_)) => panic!("cannot perform numeric operation {} for boolean arguments", stringify!($op)),
                $(
                    ([<$type >](lhs), [<  $type >](rhs)) => AnySignal::from(lhs.$op(&rhs)),
                    ([<$type >](lhs), [< Const $type >](rhs)) => AnySignal::from(lhs.$op(&rhs)),
                    ([<Const $type >](lhs), [<  $type >](rhs)) => AnySignal::from(lhs.$op(&rhs)),
                    ([<Const $type >](lhs), [< Const $type >](rhs)) => AnySignal::from(lhs.$op(&rhs)),
                )*
                _ => panic!("mismatched argument types for {} operation", stringify!($op)),
                }
            }
        }
    };

    // Binary numeric opeartions
    ($op:ident, $lhs:ident, $rhs:ident) => {
        signal_num_op_impl!(
            $op, $lhs, $rhs,
            [Int, UInt, Float]
        )
    };
}

pub(crate) use signal_num_op_impl;

/// Helper struct to evaluate a [`NumExpr`] given a trace.
pub struct NumExprEval;

impl NumExprEval {
    pub fn eval(root: &NumExpr, trace: &impl Trace) -> AnySignal {
        use core::ops::{Add, Div, Mul, Neg, Sub};

        use argus_core::signals::traits::SignalAbs;
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
                signal_num_op_impl!(neg, arg_sig, [Int, Float])
            }
            NumExpr::Add { args } => {
                let args_signals = args.iter().map(|arg| Self::eval(arg, trace));
                args_signals
                    .reduce(|acc, arg| signal_num_op_impl!(add, acc, arg))
                    .unwrap()
            }
            NumExpr::Sub { lhs, rhs } => {
                let lhs = Self::eval(lhs, trace);
                let rhs = Self::eval(rhs, trace);
                signal_num_op_impl!(sub, lhs, rhs)
            }
            NumExpr::Mul { args } => {
                let args_signals = args.iter().map(|arg| Self::eval(arg, trace));
                args_signals
                    .reduce(|acc, arg| signal_num_op_impl!(mul, acc, arg))
                    .unwrap()
            }
            NumExpr::Div { dividend, divisor } => {
                let dividend = Self::eval(dividend, trace);
                let divisor = Self::eval(divisor, trace);
                signal_num_op_impl!(div, dividend, divisor)
            }
            NumExpr::Abs { arg } => {
                let arg = Self::eval(arg, trace);
                signal_num_op_impl!(abs, arg, [Int, UInt, Float])
            }
        }
    }
}
