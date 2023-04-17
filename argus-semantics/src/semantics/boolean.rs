use argus_core::prelude::*;
use argus_core::signals::SignalPartialOrd;

use crate::eval::eval_num_expr;
use crate::{Semantics, Trace};

/// Boolean semantics of Argus expressions
pub struct BooleanSemantics;

impl Semantics for BooleanSemantics {
    type Output = Signal<bool>;
    type Context = ();

    fn eval(expr: &BoolExpr, trace: &impl Trace, ctx: Self::Context) -> ArgusResult<Self::Output> {
        match expr {
            BoolExpr::BoolLit(val) => Ok(Signal::constant(*val)),
            BoolExpr::BoolVar { name } => trace.get(name.as_str()).cloned().ok_or(ArgusError::SignalNotPresent),
            BoolExpr::Cmp { op, lhs, rhs } => {
                use argus_core::expr::Ordering::*;
                let lhs = eval_num_expr::<f64>(lhs, trace)?;
                let rhs = eval_num_expr::<f64>(rhs, trace)?;
                let ret = match op {
                    Eq => lhs.signal_eq(&rhs),
                    NotEq => lhs.signal_ne(&rhs),
                    Less { strict } if *strict => lhs.signal_lt(&rhs),
                    Less { strict: _ } => lhs.signal_le(&rhs),
                    Greater { strict } if *strict => lhs.signal_gt(&rhs),
                    Greater { strict: _ } => lhs.signal_ge(&rhs),
                };
                ret.ok_or(ArgusError::InvalidOperation)
            }
            BoolExpr::Not { arg } => {
                let arg = Self::eval(arg, trace, ctx)?;
                Ok(!&arg)
            }
            BoolExpr::And { args } => {
                let mut ret = Signal::constant(true);
                for arg in args.iter() {
                    let arg = Self::eval(arg, trace, ctx)?;
                    ret = &ret & &arg;
                }
                Ok(ret)
            }
            BoolExpr::Or { args } => {
                let mut ret = Signal::constant(false);
                for arg in args.iter() {
                    let arg = Self::eval(arg, trace, ctx)?;
                    ret = &ret | &arg;
                }
                Ok(ret)
            }
            BoolExpr::Next { arg: _ } => todo!(),
            BoolExpr::Always { arg } => {
                let mut arg = Self::eval(arg, trace, ctx)?;
                match &mut arg {
                    // if signal is empty or constant, return the signal itself.
                    // This works because if a signal is True everythere, then it must
                    // "always be true".
                    Signal::Empty | Signal::Constant { value: _ } => (),
                    Signal::Sampled { values, time_points } => {
                        // Compute the & in a expanding window fashion from the back
                        for i in (0..(time_points.len() - 1)).rev() {
                            values[i] &= values[i + 1];
                        }
                    }
                }
                Ok(arg)
            }
            BoolExpr::Eventually { arg } => {
                let mut arg = Self::eval(arg, trace, ctx)?;
                match &mut arg {
                    // if signal is empty or constant, return the signal itself.
                    // This works because if a signal is True everywhere, then it must
                    // "eventually be true", and if it is False everywhere, then it will
                    // "never be true".
                    Signal::Empty | Signal::Constant { value: _ } => (),
                    Signal::Sampled { values, time_points } => {
                        // Compute the | in a expanding window fashion from the back
                        for i in (0..(time_points.len() - 1)).rev() {
                            values[i] |= values[i + 1];
                        }
                    }
                }
                Ok(arg)
            }
            BoolExpr::Until { lhs, rhs } => todo!(),
        }
    }
}
