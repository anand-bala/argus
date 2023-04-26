use std::iter::zip;

use argus_core::expr::BoolExpr;
use argus_core::prelude::*;
use argus_core::signals::traits::{SignalAbs, SignalMinMax};
use argus_core::signals::SignalNumCast;

use crate::eval::eval_num_expr;
use crate::{Semantics, Trace};

fn top_or_bot(sig: &Signal<bool>) -> Signal<f64> {
    match sig {
        Signal::Empty => Signal::Empty,
        Signal::Constant { value } => Signal::constant(*value).to_f64().unwrap(),
        Signal::Sampled { values, time_points } => zip(time_points, values)
            .map(|(&t, &v)| if v { (t, f64::INFINITY) } else { (t, f64::NEG_INFINITY) })
            .collect(),
    }
}

/// Quantitative semantics for Argus expressions
pub struct QuantitativeSemantics;

impl Semantics for QuantitativeSemantics {
    type Output = Signal<f64>;
    type Context = ();

    fn eval(expr: &BoolExpr, trace: &impl Trace, ctx: Self::Context) -> ArgusResult<Self::Output> {
        let ret: Self::Output = match expr {
            BoolExpr::BoolLit(val) => top_or_bot(&Signal::constant(*val)),
            BoolExpr::BoolVar { name } => {
                let sig = trace.get::<bool>(name.as_str()).ok_or(ArgusError::SignalNotPresent)?;
                top_or_bot(sig)
            }
            BoolExpr::Cmp { op, lhs, rhs } => {
                use argus_core::expr::Ordering::*;
                let lhs = eval_num_expr::<f64>(lhs, trace)?;
                let rhs = eval_num_expr::<f64>(rhs, trace)?;

                match op {
                    Eq => -&((&lhs - &rhs).abs()),
                    NotEq => (&lhs - &rhs).abs(),
                    Less { strict: _ } => &rhs - &lhs,
                    Greater { strict: _ } => &lhs - &rhs,
                }
            }
            BoolExpr::Not { arg } => {
                let arg = Self::eval(arg, trace, ctx)?;
                -&arg
            }
            BoolExpr::And { args } => {
                assert!(args.len() >= 2);
                let args = args
                    .iter()
                    .map(|arg| Self::eval(arg, trace, ctx))
                    .collect::<ArgusResult<Vec<_>>>()?;
                args.into_iter()
                    .reduce(|lhs, rhs| lhs.min(&rhs))
                    .ok_or(ArgusError::InvalidOperation)?
            }
            BoolExpr::Or { args } => {
                assert!(args.len() >= 2);
                let args = args
                    .iter()
                    .map(|arg| Self::eval(arg, trace, ctx))
                    .collect::<ArgusResult<Vec<_>>>()?;
                args.into_iter()
                    .reduce(|lhs, rhs| lhs.max(&rhs))
                    .ok_or(ArgusError::InvalidOperation)?
            }
            BoolExpr::Next { arg: _ } => todo!(),
            BoolExpr::Always { arg } => {
                let mut arg = Self::eval(arg, trace, ctx)?;
                match &mut arg {
                    // if signal is empty or constant, return the signal itself.
                    // This works because if a signal is positive everywhere, then it must
                    // "always be positive" (and vice versa).
                    Signal::Empty | Signal::Constant { value: _ } => (),
                    Signal::Sampled { values, time_points } => {
                        // Compute the min in a expanding window fashion from the back
                        for i in (0..(time_points.len() - 1)).rev() {
                            values[i] = values[i].min(values[i + 1]);
                        }
                    }
                }
                arg
            }
            BoolExpr::Eventually { arg } => {
                let mut arg = Self::eval(arg, trace, ctx)?;
                match &mut arg {
                    // if signal is empty or constant, return the signal itself.
                    // This works because if a signal is positive somewhere, then it must
                    // "eventually be positive" (and vice versa).
                    Signal::Empty | Signal::Constant { value: _ } => (),
                    Signal::Sampled { values, time_points } => {
                        // Compute the max in a expanding window fashion from the back
                        for i in (0..(time_points.len() - 1)).rev() {
                            values[i] = values[i].max(values[i + 1]);
                        }
                    }
                }
                arg
            }
            BoolExpr::Until { lhs: _, rhs: _ } => todo!(),
        };
        Ok(ret)
    }
}
