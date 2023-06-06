use std::ops::Bound;
use std::time::Duration;

use argus_core::expr::{Always, And, BoolVar, Cmp, Eventually, Interval, Next, Not, Or, Oracle, Until};
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
            BoolExpr::BoolLit(val) => Ok(Signal::constant(val.0)),
            BoolExpr::BoolVar(BoolVar { name }) => {
                trace.get(name.as_str()).cloned().ok_or(ArgusError::SignalNotPresent)
            }
            BoolExpr::Cmp(Cmp { op, lhs, rhs }) => {
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
            BoolExpr::Not(Not { arg }) => {
                let arg = Self::eval(arg, trace, ctx)?;
                Ok(!&arg)
            }
            BoolExpr::And(And { args }) => {
                let mut ret = Signal::constant(true);
                for arg in args.iter() {
                    let arg = Self::eval(arg, trace, ctx)?;
                    ret = &ret & &arg;
                }
                Ok(ret)
            }
            BoolExpr::Or(Or { args }) => {
                let mut ret = Signal::constant(false);
                for arg in args.iter() {
                    let arg = Self::eval(arg, trace, ctx)?;
                    ret = &ret | &arg;
                }
                Ok(ret)
            }
            BoolExpr::Next(Next { arg }) => {
                let arg = Self::eval(arg, trace, ctx)?;
                compute_next(arg)
            }
            BoolExpr::Oracle(Oracle { steps, arg }) => {
                let arg = Self::eval(arg, trace, ctx)?;
                compute_oracle(arg, *steps)
            }
            BoolExpr::Always(Always { arg, interval }) => {
                let arg = Self::eval(arg, trace, ctx)?;
                compute_always(arg, interval)
            }
            BoolExpr::Eventually(Eventually { arg, interval }) => {
                let arg = Self::eval(arg, trace, ctx)?;
                compute_eventually(arg, interval)
            }
            BoolExpr::Until(Until { lhs, rhs, interval }) => {
                let lhs = Self::eval(lhs, trace, ctx)?;
                let rhs = Self::eval(rhs, trace, ctx)?;
                compute_until(lhs, rhs, interval)
            }
        }
    }
}

/// Compute next for a signal
fn compute_next(signal: Signal<bool>) -> ArgusResult<Signal<bool>> {
    unimplemented!()
}

/// Compute oracle for a signal
fn compute_oracle(signal: Signal<bool>, steps: usize) -> ArgusResult<Signal<bool>> {
    unimplemented!()
}

/// Compute always for a signal
fn compute_always(signal: Signal<bool>, interval: &Interval) -> ArgusResult<Signal<bool>> {
    if interval.is_empty() || interval.is_singleton() {
        return Err(ArgusError::InvalidInterval {
            reason: "interval is either empty or singleton",
        });
    }
    let ret = match signal {
        // if signal is empty or constant, return the signal itself.
        // This works because if a signal is True everythere, then it must
        // "always be true".
        sig @ (Signal::Empty | Signal::Constant { value: _ }) => sig,
        sig => {
            use Bound::*;
            if interval.is_untimed() {
                compute_untimed_always(sig)
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_always(sig, *a, Some(*b))
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_always(sig, *a, None)
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed always for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
fn compute_timed_always(signal: Signal<bool>, a: Duration, b: Option<Duration>) -> Signal<bool> {
    match b {
        Some(b) => {
            // We want to compute the
            todo!()
        }
        None => {
            // Shift the signal to the left by `a` and then run the untimed always.
            let shifted = signal.shift_left(a);
            compute_untimed_always(shifted)
        }
    }
}

/// Compute untimed always
fn compute_untimed_always(signal: Signal<bool>) -> Signal<bool> {
    let Signal::Sampled {
            mut values,
            time_points,
        } = signal
    else {
        unreachable!("we shouldn't be passing non-sampled signals here") 
    };
    // Compute the & in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] &= values[i + 1];
    }
    Signal::Sampled { values, time_points }
}

/// Compute eventually for a signal
fn compute_eventually(signal: Signal<bool>, interval: &Interval) -> ArgusResult<Signal<bool>> {
    if interval.is_empty() || interval.is_singleton() {
        return Err(ArgusError::InvalidInterval {
            reason: "interval is either empty or singleton",
        });
    }
    let ret = match signal {
        // if signal is empty or constant, return the signal itself.
        // This works because if a signal is True everythere, then it must
        // "eventually be true".
        sig @ (Signal::Empty | Signal::Constant { value: _ }) => sig,
        sig => {
            use Bound::*;
            if interval.is_singleton() {
                // for singleton intervals, return the signal itself.
                sig
            } else if interval.is_untimed() {
                compute_untimed_eventually(sig)
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_eventually(sig, *a, Some(*b))
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_eventually(sig, *a, None)
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed eventually for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
fn compute_timed_eventually(signal: Signal<bool>, a: Duration, b: Option<Duration>) -> Signal<bool> {
    match b {
        Some(b) => {
            // We want to compute the
            todo!()
        }
        None => {
            // Shift the signal to the left by `a` and then run the untimed eventually.
            let shifted = signal.shift_left(a);
            compute_untimed_eventually(shifted)
        }
    }
}

/// Compute untimed eventually
fn compute_untimed_eventually(signal: Signal<bool>) -> Signal<bool> {
    let Signal::Sampled {
            mut values,
            time_points,
        } = signal
    else {
        unreachable!("we shouldn't be passing non-sampled signals here") 
    };
    // Compute the | in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] |= values[i + 1];
    }
    Signal::Sampled { values, time_points }
}

/// Compute until
fn compute_until(lhs: Signal<bool>, rhs: Signal<bool>, interval: &Interval) -> ArgusResult<Signal<bool>> {
    let ret = match (lhs, rhs) {
        // If either signals are empty, return empty
        (sig @ Signal::Empty, _) | (_, sig @ Signal::Empty) => sig,
        (_, Signal::Constant { value: false }) => Signal::const_false(),
        (_, Signal::Constant { value: true }) => Signal::const_true(),

        (Signal::Constant { value: true }, sig) => {
            // This is the identity for eventually
            compute_eventually(sig, interval)?
        }
        (Signal::Constant { value: false }, sig) => {
            // This is the identity for next
            compute_next(sig)?
        }
        (
            lhs @ Signal::Sampled {
                values: _,
                time_points: _,
            },
            rhs @ Signal::Sampled {
                values: _,
                time_points: _,
            },
        ) => {
            use Bound::*;
            if interval.is_untimed() {
                compute_untimed_until(lhs, rhs)
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_until(lhs, rhs, *a, Some(*b))
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_until(lhs, rhs, *a, None)
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed until for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
fn compute_timed_until(lhs: Signal<bool>, rhs: Signal<bool>, a: Duration, b: Option<Duration>) -> Signal<bool> {
    // For this, we will perform the Until rewrite defined in [1]:
    //
    // $$
    //  \varphi_1 U_{[a, b]} \varphi_2 = F_{[a,b]} \varphi_2 \land (\varphi_1 U_{[a,
    // \infty)} \varphi_2)
    // $$
    //
    // $$
    //  \varphi_1 U_{[a, \infty)} \varphi_2 = G_{[0,a]} (\varphi_1 U \varphi_2)
    // $$
    //
    //
    // [1] A. Donzé, T. Ferrère, and O. Maler, "Efficient Robust Monitoring for STL." doi:
    // 10.1007/978-3-642-39799-8_19.

    match b {
        Some(b) => {
            // First compute eventually [a, b]
            let ev_a_b_rhs = compute_timed_eventually(rhs, a, Some(b));
            // Then compute until [a, \infty) (lhs, rhs)
            let unt_a_inf = compute_timed_until(lhs, rhs, a, None);
            // Then & them
            &ev_a_b_rhs & &unt_a_inf
        }
        None => {
            // First compute untimed until (lhs, rhs)
            let untimed_until = compute_untimed_until(lhs, rhs);
            // Compute G [0, a]
            compute_untimed_always(untimed_until)
        }
    }
}

/// Compute untimed until
fn compute_untimed_until(lhs: Signal<bool>, rhs: Signal<bool>) -> Signal<bool> {
    let sync_points = lhs.sync_with_intersection(&rhs);
    todo!()
}
