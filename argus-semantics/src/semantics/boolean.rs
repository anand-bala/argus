use std::ops::Bound;
use std::time::Duration;

use argus_core::expr::*;
use argus_core::prelude::*;

use crate::traits::{BooleanSemantics, QuantitativeSemantics, Trace};

impl BooleanSemantics for BoolExpr {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        match self {
            BoolExpr::BoolLit(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::BoolVar(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Cmp(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Not(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::And(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Or(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Next(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Oracle(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Always(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Eventually(sig) => BooleanSemantics::eval(sig, trace),
            BoolExpr::Until(sig) => BooleanSemantics::eval(sig, trace),
        }
    }
}

impl BooleanSemantics for BoolLit {
    fn eval(&self, _trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        Ok(Signal::constant(self.0))
    }
}

impl BooleanSemantics for BoolVar {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        trace
            .get(self.name.as_str())
            .cloned()
            .ok_or(ArgusError::SignalNotPresent)
    }
}

impl BooleanSemantics for Cmp {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        use argus_core::expr::Ordering::*;

        let lhs = QuantitativeSemantics::eval(self.lhs.as_ref(), trace)?;
        let rhs = QuantitativeSemantics::eval(self.rhs.as_ref(), trace)?;
        let ret = match self.op {
            Eq => lhs.signal_eq(&rhs),
            NotEq => lhs.signal_ne(&rhs),
            Less { strict } if *strict => lhs.signal_lt(&rhs),
            Less { strict: _ } => lhs.signal_le(&rhs),
            Greater { strict } if *strict => lhs.signal_gt(&rhs),
            Greater { strict: _ } => lhs.signal_ge(&rhs),
        };
        ret.ok_or(ArgusError::InvalidOperation)
    }
}

impl BooleanSemantics for Not {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let arg = BooleanSemantics::eval(self.arg.as_ref(), trace)?;
        Ok(arg.not())
    }
}

impl BooleanSemantics for And {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let mut ret = Signal::constant(true);
        for arg in self.args.iter() {
            let arg = Self::eval(arg, trace)?;
            ret = ret.and(&arg);
        }
    }
}

impl BooleanSemantics for Or {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let mut ret = Signal::constant(true);
        for arg in self.args.iter() {
            let arg = Self::eval(arg, trace)?;
            ret = ret.or(&arg);
        }
    }
}

fn compute_next(arg: Signal<bool>) -> ArgusResult<Signal<bool>> {
    match arg {
        Signal::Empty => Ok(Signal::Empty),
        sig @ Signal::Constant { value: _ } => {
            // Just return the signal as is
            Ok(sig)
        }
        Signal::Sampled {
            mut values,
            mut time_points,
        } => {
            // TODO(anand): Verify this
            // Just shift the signal by 1 timestamp
            assert!(values.len() == time_points.len());
            if values.len() <= 1 {
                return Ok(Signal::Empty);
            }
            values.remove(0);
            time_points.pop();
            Ok(Signal::Sampled { values, time_points })
        }
    }
}

impl BooleanSemantics for Next {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let arg = BooleanSemantics::eval(self.arg.as_ref(), trace)?;
        compute_next(arg)
    }
}

impl BooleanSemantics for Oracle {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        if self.steps == 0 {
            Ok(Signal::Empty)
        } else {
            (0..self.steps).try_fold(BooleanSemantics::eval(self.arg.as_ref(), trace)?, |arg, _| {
                compute_next(arg)
            })
        }
    }
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
            // We want to compute the windowed min/and of the signal.
            // The window is dictated by the time duration though.
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

impl BooleanSemantics for Always {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let arg = BooleanSemantics::eval(&self.arg, trace)?;
        compute_always(arg, &self.interval)
    }
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

/// Compute timed eventually for the interval `[a, b]` (or, if `b` is `None`, `[a,..]`.
fn compute_timed_eventually(signal: Signal<bool>, a: Duration, b: Option<Duration>) -> Signal<bool> {
    match b {
        Some(b) => {
            // We want to compute the windowed max/or of the signal.
            // The window is dictated by the time duration though.
            todo!()
        }
        None => {
            // Shift the signal to the left by `a` and then run the untimedeventually.
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

impl BooleanSemantics for Eventually {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        let arg = BooleanSemantics::eval(&self.arg, trace)?;
        compute_eventually(arg, &self.interval)
    }
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
    // $$
    //  \varphi_1 U_{[a, b]} \varphi_2 = F_{[a,b]} \varphi_2 \land (\varphi_1 U_{[a,
    // \infty)} \varphi_2)
    // $$
    //
    // $$
    //  \varphi_1 U_{[a, \infty)} \varphi_2 = G_{[0,a]} (\varphi_1 U \varphi_2)
    // $$
    //
    // [1] A. Donzé, T. Ferrère, and O. Maler, "Efficient Robust Monitoring for STL."

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

impl BooleanSemantics for Until {
    fn eval(&self, trace: &impl Trace) -> ArgusResult<Signal<bool>> {
        todo!()
    }
}
