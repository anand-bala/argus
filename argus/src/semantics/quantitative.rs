use std::ops::Bound;
use std::time::Duration;

use itertools::Itertools;
use num_traits::{Num, NumCast};

use super::utils::lemire_minmax::MonoWedge;
use super::Trace;
use crate::core::expr::*;
use crate::core::signals::{InterpolationMethod, SignalAbs};
use crate::{ArgusError, ArgusResult, Signal};

/// Quantitative semantics for Signal Temporal Logic expressionsd define by an [`Expr`].
pub struct QuantitativeSemantics;

impl QuantitativeSemantics {
    /// Evaluates a [Boolean expression](BoolExpr) given a [`Trace`].
    pub fn eval<I>(expr: &BoolExpr, trace: &impl Trace) -> ArgusResult<Signal<f64>>
    where
        I: InterpolationMethod<f64>,
    {
        let ret = match expr {
            BoolExpr::BoolLit(val) => top_or_bot(&Signal::constant(val.0)),
            BoolExpr::BoolVar(BoolVar { name }) => trace
                .get::<bool>(name.as_str())
                .ok_or(ArgusError::SignalNotPresent)
                .map(top_or_bot)?,
            BoolExpr::Cmp(Cmp { op, lhs, rhs }) => {
                use crate::core::expr::Ordering::*;
                let lhs = Self::eval_num_expr::<f64, I>(lhs, trace)?;
                let rhs = Self::eval_num_expr::<f64, I>(rhs, trace)?;

                match op {
                    Eq => lhs.abs_diff::<_, I>(&rhs).negate(),
                    NotEq => lhs.abs_diff::<_, I>(&rhs).negate(),
                    Less { strict: _ } => rhs.sub::<_, I>(&lhs),
                    Greater { strict: _ } => lhs.sub::<_, I>(&rhs),
                }
            }
            BoolExpr::Not(Not { arg }) => Self::eval::<I>(arg, trace)?.negate(),
            BoolExpr::And(And { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval::<I>(arg, trace)).try_fold(
                    Signal::constant(f64::INFINITY),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.min::<I>(&item))
                    },
                )?
            }
            BoolExpr::Or(Or { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval::<I>(arg, trace)).try_fold(
                    Signal::constant(f64::NEG_INFINITY),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.max::<I>(&item))
                    },
                )?
            }
            BoolExpr::Next(Next { arg }) => {
                let arg = Self::eval::<I>(arg, trace)?;
                compute_next(arg)?
            }
            BoolExpr::Oracle(Oracle { steps, arg }) => {
                let arg = Self::eval::<I>(arg, trace)?;
                compute_oracle(arg, *steps)?
            }
            BoolExpr::Always(Always { arg, interval }) => {
                let arg = Self::eval::<I>(arg, trace)?;
                compute_always::<I>(arg, interval)?
            }
            BoolExpr::Eventually(Eventually { arg, interval }) => {
                let arg = Self::eval::<I>(arg, trace)?;
                compute_eventually::<I>(arg, interval)?
            }
            BoolExpr::Until(Until { lhs, rhs, interval }) => {
                let lhs = Self::eval::<I>(lhs, trace)?;
                let rhs = Self::eval::<I>(rhs, trace)?;
                compute_until::<I>(lhs, rhs, interval)?
            }
        };
        Ok(ret)
    }

    /// Evaluates a [numeric expression](NumExpr) given a [`Trace`].
    pub fn eval_num_expr<T, I>(root: &NumExpr, trace: &impl Trace) -> ArgusResult<Signal<T>>
    where
        T: Num + NumCast + Clone + PartialOrd,
        for<'a> &'a T: core::ops::Neg<Output = T>,
        for<'a> &'a T: core::ops::Add<&'a T, Output = T>,
        for<'a> &'a T: core::ops::Sub<&'a T, Output = T>,
        for<'a> &'a T: core::ops::Mul<&'a T, Output = T>,
        for<'a> &'a T: core::ops::Div<&'a T, Output = T>,
        Signal<T>: SignalAbs,
        I: InterpolationMethod<T>,
    {
        match root {
            NumExpr::IntLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::UIntLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::FloatLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::IntVar(IntVar { name }) => trace.get::<i64>(name.as_str()).unwrap().num_cast(),
            NumExpr::UIntVar(UIntVar { name }) => trace.get::<u64>(name.as_str()).unwrap().num_cast(),
            NumExpr::FloatVar(FloatVar { name }) => trace.get::<f64>(name.as_str()).unwrap().num_cast(),
            NumExpr::Neg(Neg { arg }) => Self::eval_num_expr::<T, I>(arg, trace).map(|sig| sig.negate()),
            NumExpr::Add(Add { args }) => {
                let mut ret: Signal<T> = Signal::<T>::zero();
                for arg in args.iter() {
                    let arg = Self::eval_num_expr::<T, I>(arg, trace)?;
                    ret = ret.add::<_, I>(&arg);
                }
                Ok(ret)
            }
            NumExpr::Sub(Sub { lhs, rhs }) => {
                let lhs = Self::eval_num_expr::<T, I>(lhs, trace)?;
                let rhs = Self::eval_num_expr::<T, I>(rhs, trace)?;
                Ok(lhs.sub::<_, I>(&rhs))
            }
            NumExpr::Mul(Mul { args }) => {
                let mut ret: Signal<T> = Signal::<T>::one();
                for arg in args.iter() {
                    let arg = Self::eval_num_expr::<T, I>(arg, trace)?;
                    ret = ret.mul::<_, I>(&arg);
                }
                Ok(ret)
            }
            NumExpr::Div(Div { dividend, divisor }) => {
                let dividend = Self::eval_num_expr::<T, I>(dividend, trace)?;
                let divisor = Self::eval_num_expr::<T, I>(divisor, trace)?;
                Ok(dividend.div::<_, I>(&divisor))
            }
            NumExpr::Abs(Abs { arg }) => {
                let arg = Self::eval_num_expr::<T, I>(arg, trace)?;
                Ok(arg.abs())
            }
        }
    }
}

fn compute_next(arg: Signal<f64>) -> ArgusResult<Signal<f64>> {
    compute_oracle(arg, 1)
}

fn compute_oracle(arg: Signal<f64>, steps: usize) -> ArgusResult<Signal<f64>> {
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
fn compute_always<I: InterpolationMethod<f64>>(signal: Signal<f64>, interval: &Interval) -> ArgusResult<Signal<f64>> {
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
                compute_untimed_always::<I>(sig)?
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
fn compute_timed_always<I: InterpolationMethod<f64>>(
    signal: Signal<f64>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<f64>> {
    let z1 = signal.negate();
    let z2 = compute_timed_eventually::<I>(z1, a, b)?;
    Ok(z2.negate())
}

/// Compute untimed always
fn compute_untimed_always<I: InterpolationMethod<f64>>(signal: Signal<f64>) -> ArgusResult<Signal<f64>> {
    // Find all the points where the argument signal crosses 0
    let Some(time_points) = signal.sync_with_intersection::<I>(&Signal::constant(0.0)) else {
        unreachable!("we shouldn't be passing non-sampled signals here")
    };
    let mut values: Vec<f64> = time_points
        .iter()
        .map(|&t| signal.interpolate_at::<I>(t).unwrap())
        .collect();
    // Compute the & in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] = values[i + 1].min(values[i]);
    }
    Ok(Signal::Sampled { values, time_points })
}

/// Compute eventually for a signal
fn compute_eventually<I: InterpolationMethod<f64>>(
    signal: Signal<f64>,
    interval: &Interval,
) -> ArgusResult<Signal<f64>> {
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
                compute_untimed_eventually::<I>(sig)?
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
fn compute_timed_eventually<I: InterpolationMethod<f64>>(
    signal: Signal<f64>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<f64>> {
    let time_points = signal.time_points().unwrap().into_iter().copied().collect_vec();
    let start_time = time_points.first().copied().unwrap();
    let end_time = time_points.last().copied().unwrap();
    // Shift the signal to the left by `a`, and interpolate at the time points.
    let shifted = signal.shift_left::<I>(a);
    let signal: Signal<f64> = time_points
        .iter()
        .map(|&t| (t, shifted.interpolate_at::<I>(t).unwrap()))
        .collect();
    match b {
        Some(b) if end_time - start_time < (b - a) => {
            assert!(b > a);

            let time_points = signal
                .sync_with_intersection::<I>(&Signal::zero())
                .expect("Non-empty time points as we shouldn't be passing non-sampled signals here.");
            let time_points_plus_b = time_points.iter().map(|t| end_time.min(*t + b)).collect_vec();
            let time_points_minus_b = time_points
                .iter()
                .map(|t| start_time.max(t.saturating_sub(b)))
                .collect_vec();
            let time_points: Vec<Duration> = itertools::kmerge([time_points, time_points_plus_b, time_points_minus_b])
                .dedup()
                .collect();

            assert!(!time_points.is_empty());
            let width = b - a;
            let mut ret_vals = Signal::<f64>::with_capacity(time_points.len());

            let mut wedge = MonoWedge::<f64>::max_wedge();
            let mut j: usize = 0;
            for i in &time_points {
                let value = signal
                    .interpolate_at::<I>(*i)
                    .expect("signal should be well defined at this point");
                wedge.purge_before(i.saturating_sub(width));
                wedge.update((*i, value));
                if i >= &(time_points[0] + width) {
                    let (new_t, new_v) = wedge
                        .front()
                        .map(|(&t, &v)| (t, v))
                        .unwrap_or_else(|| panic!("wedge should have at least 1 element"));
                    ret_vals.push(new_t, new_v)?;
                    j += 1;
                }
            }
            // Get the rest of the values
            for i in &time_points[j..] {
                wedge.purge_before(*i);
                let (t, val) = wedge
                    .front()
                    .map(|(&t, &v)| (t, v))
                    .unwrap_or_else(|| panic!("wedge should have at least 1 element"));
                assert_eq!(
                    t, *i,
                    "{:?} != {:?}\n\ttime_points = {:?}, wedge.time_points = {:?}\n\tret_vals = {:?}",
                    t, i, time_points, wedge.time_points, ret_vals,
                );
                ret_vals.push(*i, val)?;
            }
            Ok(ret_vals)
        }
        _ => compute_untimed_eventually::<I>(shifted),
    }
}

/// Compute untimed eventually
fn compute_untimed_eventually<I: InterpolationMethod<f64>>(signal: Signal<f64>) -> ArgusResult<Signal<f64>> {
    // Find all the points where the argument signal crosses 0
    let Some(time_points) = signal.sync_with_intersection::<I>(&Signal::constant(0.0)) else {
        unreachable!("we shouldn't be passing non-sampled signals here")
    };
    let mut values: Vec<f64> = time_points
        .iter()
        .map(|&t| signal.interpolate_at::<I>(t).unwrap())
        .collect();
    // Compute the | in a expanding window fashion from the back
    for i in (0..(time_points.len() - 1)).rev() {
        values[i] = values[i + 1].max(values[i]);
    }
    Ok(Signal::Sampled { values, time_points })
}

/// Compute until
fn compute_until<I: InterpolationMethod<f64>>(
    lhs: Signal<f64>,
    rhs: Signal<f64>,
    interval: &Interval,
) -> ArgusResult<Signal<f64>> {
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
fn compute_timed_until<I: InterpolationMethod<f64>>(
    lhs: Signal<f64>,
    rhs: Signal<f64>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<f64>> {
    // First compute eventually [a, b]
    let ev_a_b_rhs = compute_timed_eventually::<I>(rhs.clone(), a, b)?;
    // Then compute untimed until
    let untimed = compute_untimed_until::<I>(lhs, rhs)?;
    if a.is_zero() {
        Ok(ev_a_b_rhs.min::<I>(&untimed))
    } else {
        let g_a = compute_timed_always::<I>(untimed, Duration::ZERO, Some(a))?;
        Ok(ev_a_b_rhs.min::<I>(&g_a))
    }
}

/// Compute untimed until
fn compute_untimed_until<I: InterpolationMethod<f64>>(lhs: Signal<f64>, rhs: Signal<f64>) -> ArgusResult<Signal<f64>> {
    let sync_points = lhs.sync_with_intersection::<I>(&rhs).unwrap();
    let mut ret_samples = Vec::with_capacity(sync_points.len());
    let expected_len = sync_points.len();

    let mut next = f64::NEG_INFINITY;

    for (i, t) in sync_points.into_iter().enumerate().rev() {
        let v1 = lhs.interpolate_at::<I>(t).unwrap();
        let v2 = rhs.interpolate_at::<I>(t).unwrap();

        let z = f64::max(f64::min(v1, v2), f64::min(v1, next));
        if z == next && i < (expected_len - 2) {
            ret_samples.pop();
        }
        ret_samples.push((t, z));
        next = z;
    }

    Signal::<f64>::try_from_iter(ret_samples.into_iter().rev())
}

fn top_or_bot(sig: &Signal<bool>) -> Signal<f64> {
    let bool2float = |&v| {
        if v {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    };
    match sig {
        Signal::Empty => Signal::Empty,
        Signal::Constant { value } => Signal::constant(bool2float(value)),
        Signal::Sampled { values, time_points } => {
            time_points.iter().copied().zip(values.iter().map(bool2float)).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::zip;
    use std::time::Duration;

    use itertools::assert_equal;

    use super::*;
    use crate::core::expr::ExprBuilder;
    use crate::core::signals::interpolation::{Constant, Linear};
    use crate::core::signals::AnySignal;

    const FLOAT_EPS: f64 = 1.0e-8;

    fn assert_approx_eq(lhs: &Signal<f64>, rhs: &Signal<f64>) {
        zip(lhs, rhs).enumerate().for_each(|(i, (s1, s2))| {
            assert_eq!(
                s1.0, s2.0,
                "Failed assertion {:?} != {:?} for iteration {}",
                s1.0, s2.0, i
            );
            assert!(
                (s2.1 - s1.1).abs() <= FLOAT_EPS,
                "Failed approx equal assertion: {} != {} for iteration {}",
                s1.1,
                s2.1,
                i
            );
        });
    }

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
    fn num_constant() {
        let expr_builder = ExprBuilder::new();

        let spec = expr_builder.float_const(5.0);
        let trace = MyTrace::default();

        let robustness = QuantitativeSemantics::eval_num_expr::<f64, Linear>(&spec, &trace).unwrap();

        assert!(matches!(robustness, Signal::Constant { value } if value == 5.0));
    }

    #[test]
    fn addition() {
        let mut ctx = ExprBuilder::new();

        let a = Box::new(ctx.float_var("a".to_owned()).unwrap());
        let b = Box::new(ctx.float_var("b".to_owned()).unwrap());
        let spec = ctx.make_add([*a, *b]).unwrap();

        {
            let signals = HashMap::from_iter(vec![
                (
                    "a".to_owned(),
                    Box::new(Signal::from_iter(vec![
                        (Duration::from_secs_f64(0.0), 1.3),
                        (Duration::from_secs_f64(0.7), 3.0),
                        (Duration::from_secs_f64(1.3), 0.1),
                        (Duration::from_secs_f64(2.1), -2.2),
                    ])) as Box<dyn AnySignal>,
                ),
                (
                    "b".to_owned(),
                    Box::new(Signal::from_iter(vec![
                        (Duration::from_secs_f64(0.0), 2.5),
                        (Duration::from_secs_f64(0.7), 4.0),
                        (Duration::from_secs_f64(1.3), -1.2),
                        (Duration::from_secs_f64(2.1), 1.7),
                    ])) as Box<dyn AnySignal>,
                ),
            ]);

            let trace = MyTrace { signals };

            let rob = QuantitativeSemantics::eval_num_expr::<f64, Linear>(&spec, &trace).unwrap();
            let expected = Signal::from_iter(vec![
                (Duration::from_secs_f64(0.0), 1.3 + 2.5),
                (Duration::from_secs_f64(0.7), 3.0 + 4.0),
                (Duration::from_secs_f64(1.3), 0.1 + -1.2),
                (Duration::from_secs_f64(2.1), -2.2 + 1.7),
            ]);

            assert_equal(&rob, &expected);
        }
        {
            let signals = HashMap::from_iter(vec![
                (
                    "a".to_owned(),
                    Box::new(Signal::from_iter(vec![
                        (Duration::from_secs_f64(0.0), 1.3),
                        (Duration::from_secs_f64(0.7), 3.0),
                        (Duration::from_secs_f64(1.3), 4.0),
                        (Duration::from_secs_f64(2.1), 3.0),
                    ])) as Box<dyn AnySignal>,
                ),
                (
                    "b".to_owned(),
                    Box::new(Signal::from_iter(vec![
                        (Duration::from_secs_f64(0.0), 2.5),
                        (Duration::from_secs_f64(0.7), 4.0),
                        (Duration::from_secs_f64(1.3), 3.0),
                        (Duration::from_secs_f64(2.1), 4.0),
                    ])) as Box<dyn AnySignal>,
                ),
            ]);

            let trace = MyTrace { signals };

            let rob = QuantitativeSemantics::eval_num_expr::<f64, Linear>(&spec, &trace).unwrap();
            let expected = Signal::from_iter(vec![
                (Duration::from_secs_f64(0.0), 1.3 + 2.5),
                (Duration::from_secs_f64(0.7), 3.0 + 4.0),
                (Duration::from_secs_f64(1.3), 4.0 + 3.0),
                (Duration::from_secs_f64(2.1), 3.0 + 4.0),
            ]);

            assert_equal(&rob, &expected);
        }
    }

    #[test]
    fn less_than() {
        let mut ctx = ExprBuilder::new();

        let a = Box::new(ctx.float_var("a".to_owned()).unwrap());
        let spec = Box::new(ctx.make_lt(a, Box::new(ctx.float_const(0.0))));

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

        let rob = QuantitativeSemantics::eval::<Linear>(&spec, &trace).unwrap();
        let expected = Signal::from_iter(vec![
            (Duration::from_secs_f64(0.0), 0.0 - 1.3),
            (Duration::from_secs_f64(0.7), 0.0 - 3.0),
            (Duration::from_secs_f64(1.3), 0.0 - 0.1),
            (Duration::from_secs_f64(1.334782609), 0.0), // interpolated at
            (Duration::from_secs_f64(2.1), 0.0 - (-2.2)),
        ]);

        assert_approx_eq(&rob, &expected);
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
            let rob = QuantitativeSemantics::eval::<Linear>(&spec, &trace).unwrap();
            println!("{:#?}", rob);
            let expected = Signal::from_iter(vec![
                (Duration::from_secs_f64(0.0), 4.0),
                (Duration::from_secs_f64(0.7), 4.0),
                (Duration::from_secs_f64(1.18), 1.7),
                (Duration::from_secs_f64(1.3), 1.7),
                (Duration::from_secs_f64(1.596296296), 1.7), // interpolated at
                (Duration::from_secs_f64(2.1), 1.7),
            ]);

            assert_equal(&rob, &expected);
        }
    }

    #[test]
    fn unbounded_until() {
        let mut ctx = ExprBuilder::new();

        let a = Box::new(ctx.int_var("a".to_owned()).unwrap());
        let b = Box::new(ctx.int_var("b".to_owned()).unwrap());
        let lhs = Box::new(ctx.make_gt(a, Box::new(ctx.int_const(0))));
        let rhs = Box::new(ctx.make_gt(b, Box::new(ctx.int_const(0))));
        let spec = ctx.make_until(lhs, rhs);

        {
            let signals = HashMap::from_iter(vec![
                (
                    "a".to_owned(),
                    Box::new(Signal::<i64>::from_iter(vec![
                        (Duration::from_secs(0), 2),
                        (Duration::from_secs(5), 2),
                    ])) as Box<dyn AnySignal>,
                ),
                (
                    "b".to_owned(),
                    Box::new(Signal::<i64>::from_iter(vec![
                        (Duration::from_secs(0), 4),
                        (Duration::from_secs(5), 4),
                    ])) as Box<dyn AnySignal>,
                ),
            ]);

            let trace = MyTrace { signals };
            let rob = QuantitativeSemantics::eval::<Constant>(&spec, &trace).unwrap();

            let expected = Signal::from_iter(vec![(Duration::from_secs(0), 2), (Duration::from_secs(5), 2)])
                .num_cast::<f64>()
                .unwrap();

            assert_equal(&rob, &expected);
        }

        {
            let signals = HashMap::from_iter(vec![
                (
                    "a".to_owned(),
                    Box::new(Signal::<i64>::from_iter(vec![
                        (Duration::from_secs_f64(1.0), 1),
                        (Duration::from_secs_f64(3.5), 7),
                        (Duration::from_secs_f64(4.7), 3),
                        (Duration::from_secs_f64(5.3), 5),
                        (Duration::from_secs_f64(6.2), 1),
                    ])) as Box<dyn AnySignal>,
                ),
                (
                    "b".to_owned(),
                    Box::new(Signal::<i64>::from_iter(vec![
                        (Duration::from_secs(4), 2),
                        (Duration::from_secs(6), 3),
                    ])) as Box<dyn AnySignal>,
                ),
            ]);

            let trace = MyTrace { signals };
            let rob = QuantitativeSemantics::eval::<Constant>(&spec, &trace).unwrap();

            let expected = Signal::from_iter(vec![(Duration::from_secs(4), 3), (Duration::from_secs(6), 3)])
                .num_cast::<f64>()
                .unwrap();

            assert_equal(&rob, &expected);
        }
    }
}
