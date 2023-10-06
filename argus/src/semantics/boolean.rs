use std::ops::Bound;
use std::time::Duration;

use super::utils::lemire_minmax::MonoWedge;
use super::Trace;
use crate::core::expr::*;
use crate::core::signals::{InterpolationMethod, SignalPartialOrd};
use crate::semantics::QuantitativeSemantics;
use crate::{ArgusError, ArgusResult, Signal};

/// Boolean semantics for Signal Temporal Logic expressionsd define by an [`Expr`].
pub struct BooleanSemantics;

impl BooleanSemantics {
    /// Evaluates a [Boolean expression](BoolExpr) given a [`Trace`].
    pub fn eval<BoolI, NumI>(expr: &BoolExpr, trace: &impl Trace) -> ArgusResult<Signal<bool>>
    where
        BoolI: InterpolationMethod<bool>,
        NumI: InterpolationMethod<f64>,
    {
        let ret = match expr {
            BoolExpr::BoolLit(val) => Signal::constant(val.0),
            BoolExpr::BoolVar(BoolVar { name }) => trace
                .get::<bool>(name.as_str())
                .ok_or(ArgusError::SignalNotPresent)?
                .clone(),
            BoolExpr::Cmp(Cmp { op, lhs, rhs }) => {
                use crate::core::expr::Ordering::*;
                let lhs = QuantitativeSemantics::eval_num_expr::<f64, NumI>(lhs, trace)?;
                let rhs = QuantitativeSemantics::eval_num_expr::<f64, NumI>(rhs, trace)?;

                match op {
                    Eq => lhs.signal_eq::<NumI>(&rhs).unwrap(),
                    NotEq => lhs.signal_ne::<NumI>(&rhs).unwrap(),
                    Less { strict } if *strict => lhs.signal_lt::<NumI>(&rhs).unwrap(),
                    Less { strict: _ } => lhs.signal_le::<NumI>(&rhs).unwrap(),
                    Greater { strict } if *strict => lhs.signal_gt::<NumI>(&rhs).unwrap(),
                    Greater { strict: _ } => lhs.signal_ge::<NumI>(&rhs).unwrap(),
                }
            }
            BoolExpr::Not(Not { arg }) => {
                let arg = Self::eval::<BoolI, NumI>(arg, trace)?;
                !&arg
            }
            BoolExpr::And(And { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval::<BoolI, NumI>(arg, trace)).try_fold(
                    Signal::const_true(),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.and::<BoolI>(&item))
                    },
                )?
            }
            BoolExpr::Or(Or { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval::<BoolI, NumI>(arg, trace)).try_fold(
                    Signal::const_true(),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.or::<BoolI>(&item))
                    },
                )?
            }
            BoolExpr::Next(Next { arg }) => {
                let arg = Self::eval::<BoolI, NumI>(arg, trace)?;
                compute_next(arg)?
            }
            BoolExpr::Oracle(Oracle { steps, arg }) => {
                let arg = Self::eval::<BoolI, NumI>(arg, trace)?;
                compute_oracle(arg, *steps)?
            }
            BoolExpr::Always(Always { arg, interval }) => {
                let arg = Self::eval::<BoolI, NumI>(arg, trace)?;
                compute_always::<BoolI>(arg, interval)?
            }
            BoolExpr::Eventually(Eventually { arg, interval }) => {
                let arg = Self::eval::<BoolI, NumI>(arg, trace)?;
                compute_eventually::<BoolI>(arg, interval)?
            }
            BoolExpr::Until(Until { lhs, rhs, interval }) => {
                let lhs = Self::eval::<BoolI, NumI>(lhs, trace)?;
                let rhs = Self::eval::<BoolI, NumI>(rhs, trace)?;
                compute_until::<BoolI>(lhs, rhs, interval)?
            }
        };
        Ok(ret)
    }
}

fn compute_next(arg: Signal<bool>) -> ArgusResult<Signal<bool>> {
    compute_oracle(arg, 1)
}

fn compute_oracle(arg: Signal<bool>, steps: usize) -> ArgusResult<Signal<bool>> {
    if steps == 0 {
        return Ok(Signal::Empty);
    }
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
            // Just shift the signal by `steps` timestamps
            assert_eq!(values.len(), time_points.len());
            if values.len() <= steps {
                return Ok(Signal::Empty);
            }
            let expected_len = values.len() - steps;
            let values = values.split_off(steps);
            let _ = time_points.split_off(steps);

            assert_eq!(values.len(), expected_len);
            assert_eq!(values.len(), time_points.len());
            Ok(Signal::Sampled { values, time_points })
        }
    }
}

/// Compute always for a signal
fn compute_always<I: InterpolationMethod<bool>>(
    signal: Signal<bool>,
    interval: &Interval,
) -> ArgusResult<Signal<bool>> {
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
            if interval.is_singleton() {
                // for singleton intervals, return the signal itself.
                sig
            } else if interval.is_untimed() {
                compute_untimed_always(sig)?
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_always::<I>(sig, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_always::<I>(sig, *a, None)?
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed always for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
fn compute_timed_always<I: InterpolationMethod<bool>>(
    signal: Signal<bool>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<bool>> {
    let z1 = !signal;
    let z2 = compute_timed_eventually::<I>(z1, a, b)?;
    Ok(!z2)
}

/// Compute untimed always
fn compute_untimed_always(signal: Signal<bool>) -> ArgusResult<Signal<bool>> {
    let Signal::Sampled {
        mut values,
        time_points,
    } = signal
    else {
        unreachable!("we shouldn't be passing non-sampled signals here")
    };
    // Compute the & in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] = values[i + 1].min(values[i]);
    }
    Ok(Signal::Sampled { values, time_points })
}

/// Compute eventually for a signal
fn compute_eventually<I: InterpolationMethod<bool>>(
    signal: Signal<bool>,
    interval: &Interval,
) -> ArgusResult<Signal<bool>> {
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
                compute_untimed_eventually(sig)?
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_eventually::<I>(sig, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_eventually::<I>(sig, *a, None)?
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed eventually for the interval `[a, b]` (or, if `b` is `None`, `[a,..]`.
fn compute_timed_eventually<I: InterpolationMethod<bool>>(
    signal: Signal<bool>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<bool>> {
    match b {
        Some(b) => {
            // We want to compute the windowed max/or of the signal.
            // The window is dictated by the time duration though.
            let Signal::Sampled { values, time_points } = signal else {
                unreachable!("we shouldn't be passing non-sampled signals here")
            };
            assert!(b > a);
            assert!(!time_points.is_empty());
            let signal_duration = *time_points.last().unwrap() - *time_points.first().unwrap();
            let width = if signal_duration < (b - a) {
                signal_duration
            } else {
                b - a
            };
            let mut ret_vals = Vec::with_capacity(values.len());

            // For boolean signals we dont need to worry about intersections with ZERO as much as
            // for quantitative signals, as linear interpolation is just a discrte switch.
            let mut wedge = MonoWedge::<bool>::max_wedge(width);
            for (i, value) in time_points.iter().zip(&values) {
                wedge.update((i, value));
                if i >= &(time_points[0] + width) {
                    ret_vals.push(
                        wedge
                            .front()
                            .map(|(&t, &v)| (t, v))
                            .unwrap_or_else(|| panic!("wedge should have at least 1 element")),
                    )
                }
            }
            Signal::try_from_iter(ret_vals)
        }
        None => {
            // Shift the signal to the left by `a` and then run the untimed eventually.
            let shifted = signal.shift_left::<I>(a);
            compute_untimed_eventually(shifted)
        }
    }
}

/// Compute untimed eventually
fn compute_untimed_eventually(signal: Signal<bool>) -> ArgusResult<Signal<bool>> {
    let Signal::Sampled {
        mut values,
        time_points,
    } = signal
    else {
        unreachable!("we shouldn't be passing non-sampled signals here")
    };
    // Compute the | in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] = values[i + 1].max(values[i]);
    }
    Ok(Signal::Sampled { values, time_points })
}

/// Compute until
fn compute_until<I: InterpolationMethod<bool>>(
    lhs: Signal<bool>,
    rhs: Signal<bool>,
    interval: &Interval,
) -> ArgusResult<Signal<bool>> {
    let ret = match (lhs, rhs) {
        // If either signals are empty, return empty
        (sig @ Signal::Empty, _) | (_, sig @ Signal::Empty) => sig,
        (lhs, rhs) => {
            use Bound::*;
            if interval.is_untimed() {
                compute_untimed_until::<I>(lhs, rhs)?
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_until::<I>(lhs, rhs, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_until::<I>(lhs, rhs, *a, None)?
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed until for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
///
/// For this, we will perform the Until rewrite defined in [1]:
/// $$
///  \varphi_1 U_{[a, b]} \varphi_2 = F_{[a,b]} \varphi_2 \land (\varphi_1 U_{[a,
/// \infty)} \varphi_2)
/// $$
///
/// $$
///  \varphi_1 U_{[a, \infty)} \varphi_2 = G_{[0,a]} (\varphi_1 U \varphi_2)
/// $$
///
/// [1]: <> (A. Donzé, T. Ferrère, and O. Maler, "Efficient Robust Monitoring for STL.")
fn compute_timed_until<I: InterpolationMethod<bool>>(
    lhs: Signal<bool>,
    rhs: Signal<bool>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<bool>> {
    match b {
        Some(b) => {
            // First compute eventually [a, b]
            let ev_a_b_rhs = compute_timed_eventually::<I>(rhs.clone(), a, Some(b))?;
            // Then compute until [a, \infty) (lhs, rhs)
            let unt_a_inf = compute_timed_until::<I>(lhs, rhs, a, None)?;
            // Then & them
            Ok(ev_a_b_rhs.and::<I>(&unt_a_inf))
        }
        None => {
            assert_ne!(a, Duration::ZERO, "untimed case wasn't handled for Until");
            // First compute untimed until (lhs, rhs)
            let untimed_until = compute_untimed_until::<I>(lhs, rhs)?;
            // Compute G [0, a]
            compute_timed_always::<I>(untimed_until, Duration::ZERO, Some(a))
        }
    }
}

/// Compute untimed until
fn compute_untimed_until<I: InterpolationMethod<bool>>(
    lhs: Signal<bool>,
    rhs: Signal<bool>,
) -> ArgusResult<Signal<bool>> {
    let sync_points = lhs.sync_with_intersection::<I>(&rhs).unwrap();
    let mut ret_samples = Vec::with_capacity(sync_points.len());
    let expected_len = sync_points.len();

    let mut next = false;

    for (i, t) in sync_points.into_iter().enumerate().rev() {
        let v1 = lhs.interpolate_at::<I>(t).unwrap();
        let v2 = rhs.interpolate_at::<I>(t).unwrap();

        #[allow(clippy::nonminimal_bool)]
        let z = (v1 && v2) || (v1 && next);
        if z == next && i < (expected_len - 2) {
            ret_samples.pop();
        }
        ret_samples.push((t, z));
        next = z;
    }

    Signal::<bool>::try_from_iter(ret_samples.into_iter().rev())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use itertools::assert_equal;

    use super::*;
    use crate::core::expr::ExprBuilder;
    use crate::core::signals::interpolation::Linear;
    use crate::core::signals::AnySignal;

    #[derive(Default)]
    struct MyTrace {
        signals: HashMap<String, Box<dyn AnySignal>>,
    }

    impl Trace for MyTrace {
        fn signal_names(&self) -> Vec<&str> {
            self.signals.keys().map(|key| key.as_str()).collect()
        }

        fn get<T: 'static>(&self, name: &str) -> Option<&Signal<T>> {
            let signal = self.signals.get(name)?;
            signal.as_any().downcast_ref::<Signal<T>>()
        }
    }

    #[test]
    fn less_than() {
        let mut ctx = ExprBuilder::new();

        let a = Box::new(ctx.float_var("a".to_owned()).unwrap());
        let spec = ctx.make_lt(a, Box::new(ctx.float_const(0.0)));

        let signals = HashMap::from_iter(vec![(
            "a".to_owned(),
            Box::new(Signal::from_iter(vec![
                (Duration::from_secs_f64(0.0), 1.3),
                (Duration::from_secs_f64(0.7), 3.0),
                (Duration::from_secs_f64(1.3), 0.1),
                (Duration::from_secs_f64(2.1), -2.2),
            ])) as Box<dyn AnySignal>,
        )]);

        let trace = MyTrace { signals };

        let rob = BooleanSemantics::eval::<Linear, Linear>(&spec, &trace).unwrap();
        let expected = Signal::from_iter(vec![
            (Duration::from_secs_f64(0.0), false),
            (Duration::from_secs_f64(0.7), false),
            (Duration::from_secs_f64(1.3), false),
            (Duration::from_secs_f64(1.334782609), true), // interpolated at
            (Duration::from_secs_f64(2.1), true),
        ]);

        assert_equal(&rob, &expected);
    }

    #[test]
    fn eventually_unbounded() {
        let mut ctx = ExprBuilder::new();

        let a = Box::new(ctx.float_var("a".to_owned()).unwrap());
        let cmp = Box::new(ctx.make_ge(a, Box::new(ctx.float_const(0.0))));
        let spec = ctx.make_eventually(cmp);

        {
            let signals = HashMap::from_iter(vec![(
                "a".to_owned(),
                Box::new(Signal::from_iter(vec![
                    (Duration::from_secs_f64(0.0), 2.5),
                    (Duration::from_secs_f64(0.7), 4.0),
                    (Duration::from_secs_f64(1.3), -1.0),
                    (Duration::from_secs_f64(2.1), 1.7),
                ])) as Box<dyn AnySignal>,
            )]);

            let trace = MyTrace { signals };
            let rob = BooleanSemantics::eval::<Linear, Linear>(&spec, &trace).unwrap();

            let Signal::Sampled { values, time_points: _ } = rob else {
                panic!("boolean semantics should remain sampled");
            };
            assert!(values.into_iter().all(|v| v));
        }
        {
            let signals = HashMap::from_iter(vec![(
                "a".to_owned(),
                Box::new(Signal::from_iter(vec![
                    (Duration::from_secs_f64(0.0), 2.5),
                    (Duration::from_secs_f64(0.7), 4.0),
                    (Duration::from_secs_f64(1.3), 1.7),
                    (Duration::from_secs_f64(1.4), 0.0),
                    (Duration::from_secs_f64(2.1), -2.0),
                ])) as Box<dyn AnySignal>,
            )]);

            let trace = MyTrace { signals };
            let rob = BooleanSemantics::eval::<Linear, Linear>(&spec, &trace).unwrap();
            println!("{:#?}", rob);

            let Signal::Sampled { values, time_points: _ } = rob else {
                panic!("boolean semantics should remain sampled");
            };
            assert!(values[..values.len() - 1].iter().all(|&v| v));
            assert!(!values[values.len() - 1]);
        }
    }
}
