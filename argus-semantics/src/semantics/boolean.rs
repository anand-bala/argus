use argus_core::{expr::BoolExpr, prelude::*};

use crate::{
    eval::NumExprEval,
    utils::{signal_bool_op_impl, signal_cmp_op_impl},
    Semantics, Trace,
};

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
