use argus_core::expr::NumExpr;
use argus_core::signals::{AnySignal, ConstantSignal};

use crate::utils::signal_num_op_impl;
use crate::Trace;

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
