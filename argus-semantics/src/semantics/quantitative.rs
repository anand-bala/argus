use std::ops::Bound;
use std::time::Duration;

use argus_core::expr::*;
use argus_core::prelude::*;
use argus_core::signals::interpolation::Linear;
use argus_core::signals::SignalAbs;
use num_traits::{Num, NumCast};

use crate::traits::Trace;
use crate::utils::lemire_minmax::MonoWedge;

pub struct QuantitativeSemantics;

impl QuantitativeSemantics {
    pub fn eval(expr: &BoolExpr, trace: &impl Trace) -> ArgusResult<Signal<f64>> {
        let ret = match expr {
            BoolExpr::BoolLit(val) => top_or_bot(&Signal::constant(val.0)),
            BoolExpr::BoolVar(BoolVar { name }) => trace
                .get::<bool>(name.as_str())
                .ok_or(ArgusError::SignalNotPresent)
                .map(top_or_bot)?,
            BoolExpr::Cmp(Cmp { op, lhs, rhs }) => {
                use argus_core::expr::Ordering::*;
                let lhs = Self::eval_num_expr::<f64>(lhs, trace)?;
                let rhs = Self::eval_num_expr::<f64>(rhs, trace)?;

                match op {
                    Eq => -&((&lhs - &rhs).abs()),
                    NotEq => (&lhs - &rhs).abs(),
                    Less { strict: _ } => &rhs - &lhs,
                    Greater { strict: _ } => &lhs - &rhs,
                }
            }
            BoolExpr::Not(Not { arg }) => {
                let arg = Self::eval(arg, trace)?;
                -&arg
            }
            BoolExpr::And(And { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval(arg, trace)).try_fold(
                    Signal::constant(f64::INFINITY),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.min(&item))
                    },
                )?
            }
            BoolExpr::Or(Or { args }) => {
                assert!(args.len() >= 2);
                args.iter().map(|arg| Self::eval(arg, trace)).try_fold(
                    Signal::constant(f64::NEG_INFINITY),
                    |acc, item| {
                        let item = item?;
                        Ok(acc.max(&item))
                    },
                )?
            }
            BoolExpr::Next(Next { arg }) => {
                let arg = Self::eval(arg, trace)?;
                compute_next(arg)?
            }
            BoolExpr::Oracle(Oracle { steps, arg }) => {
                let arg = Self::eval(arg, trace)?;
                compute_oracle(arg, *steps)?
            }
            BoolExpr::Always(Always { arg, interval }) => {
                let arg = Self::eval(arg, trace)?;
                compute_always(arg, interval)?
            }
            BoolExpr::Eventually(Eventually { arg, interval }) => {
                let arg = Self::eval(arg, trace)?;
                compute_eventually(arg, interval)?
            }
            BoolExpr::Until(Until { lhs, rhs, interval }) => {
                let lhs = Self::eval(lhs, trace)?;
                let rhs = Self::eval(rhs, trace)?;
                compute_until(lhs, rhs, interval)?
            }
        };
        Ok(ret)
    }

    pub fn eval_num_expr<T>(root: &NumExpr, trace: &impl Trace) -> ArgusResult<Signal<T>>
    where
        T: Num + NumCast,
        for<'a> &'a Signal<T>: std::ops::Neg<Output = Signal<T>>,
        for<'a> &'a Signal<T>: std::ops::Add<&'a Signal<T>, Output = Signal<T>>,
        for<'a> &'a Signal<T>: std::ops::Sub<&'a Signal<T>, Output = Signal<T>>,
        for<'a> &'a Signal<T>: std::ops::Mul<&'a Signal<T>, Output = Signal<T>>,
        for<'a> &'a Signal<T>: std::ops::Div<&'a Signal<T>, Output = Signal<T>>,
        Signal<T>: SignalAbs,
    {
        match root {
            NumExpr::IntLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::UIntLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::FloatLit(val) => Signal::constant(val.0).num_cast(),
            NumExpr::IntVar(IntVar { name }) => trace.get::<i64>(name.as_str()).unwrap().num_cast(),
            NumExpr::UIntVar(UIntVar { name }) => trace.get::<u64>(name.as_str()).unwrap().num_cast(),
            NumExpr::FloatVar(FloatVar { name }) => trace.get::<f64>(name.as_str()).unwrap().num_cast(),
            NumExpr::Neg(Neg { arg }) => Self::eval_num_expr::<T>(arg, trace).map(|sig| -&sig),
            NumExpr::Add(Add { args }) => {
                let mut ret: Signal<T> = Signal::<T>::zero();
                for arg in args.iter() {
                    let arg = Self::eval_num_expr::<T>(arg, trace)?;
                    ret = &ret + &arg;
                }
                Ok(ret)
            }
            NumExpr::Sub(Sub { lhs, rhs }) => {
                let lhs = Self::eval_num_expr::<T>(lhs, trace)?;
                let rhs = Self::eval_num_expr::<T>(rhs, trace)?;
                Ok(&lhs - &rhs)
            }
            NumExpr::Mul(Mul { args }) => {
                let mut ret: Signal<T> = Signal::<T>::one();
                for arg in args.iter() {
                    let arg = Self::eval_num_expr::<T>(arg, trace)?;
                    ret = &ret * &arg;
                }
                Ok(ret)
            }
            NumExpr::Div(Div { dividend, divisor }) => {
                let dividend = Self::eval_num_expr::<T>(dividend, trace)?;
                let divisor = Self::eval_num_expr::<T>(divisor, trace)?;
                Ok(&dividend / &divisor)
            }
            NumExpr::Abs(Abs { arg }) => {
                let arg = Self::eval_num_expr::<T>(arg, trace)?;
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
fn compute_always(signal: Signal<f64>, interval: &Interval) -> ArgusResult<Signal<f64>> {
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
                compute_timed_always(sig, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_always(sig, *a, None)?
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed always for the interval `[a, b]` (or, if `b` is `None`, `[a, ..]`.
fn compute_timed_always(signal: Signal<f64>, a: Duration, b: Option<Duration>) -> ArgusResult<Signal<f64>> {
    let z1 = -signal;
    let z2 = compute_timed_eventually(z1, a, b)?;
    Ok(-z2)
}

/// Compute untimed always
fn compute_untimed_always(signal: Signal<f64>) -> ArgusResult<Signal<f64>> {
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
fn compute_eventually(signal: Signal<f64>, interval: &Interval) -> ArgusResult<Signal<f64>> {
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
                compute_timed_eventually(sig, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_eventually(sig, *a, None)?
            } else {
                unreachable!("interval should be created using Interval::new, and is_untimed checks this")
            }
        }
    };
    Ok(ret)
}

/// Compute timed eventually for the interval `[a, b]` (or, if `b` is `None`, `[a,..]`.
fn compute_timed_eventually(signal: Signal<f64>, a: Duration, b: Option<Duration>) -> ArgusResult<Signal<f64>> {
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
            let mut wedge = MonoWedge::<f64>::max_wedge(width);
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
            let shifted = signal.shift_left(a);
            compute_untimed_eventually(shifted)
        }
    }
}

/// Compute untimed eventually
fn compute_untimed_eventually(signal: Signal<f64>) -> ArgusResult<Signal<f64>> {
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
fn compute_until(lhs: Signal<f64>, rhs: Signal<f64>, interval: &Interval) -> ArgusResult<Signal<f64>> {
    let ret = match (lhs, rhs) {
        // If either signals are empty, return empty
        (sig @ Signal::Empty, _) | (_, sig @ Signal::Empty) => sig,
        (lhs, rhs) => {
            use Bound::*;
            if interval.is_untimed() {
                compute_untimed_until(lhs, rhs)?
            } else if let (Included(a), Included(b)) = interval.into() {
                compute_timed_until(lhs, rhs, *a, Some(*b))?
            } else if let (Included(a), Unbounded) = interval.into() {
                compute_timed_until(lhs, rhs, *a, None)?
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
fn compute_timed_until(
    lhs: Signal<f64>,
    rhs: Signal<f64>,
    a: Duration,
    b: Option<Duration>,
) -> ArgusResult<Signal<f64>> {
    match b {
        Some(b) => {
            // First compute eventually [a, b]
            let ev_a_b_rhs = compute_timed_eventually(rhs.clone(), a, Some(b))?;
            // Then compute until [a, \infty) (lhs, rhs)
            let unt_a_inf = compute_timed_until(lhs, rhs, a, None)?;
            // Then & them
            Ok(ev_a_b_rhs.min(&unt_a_inf))
        }
        None => {
            assert_ne!(a, Duration::ZERO, "untimed case wasn't handled for Until");
            // First compute untimed until (lhs, rhs)
            let untimed_until = compute_untimed_until(lhs, rhs)?;
            // Compute G [0, a]
            compute_timed_always(untimed_until, Duration::ZERO, Some(a))
        }
    }
}

/// Compute untimed until
fn compute_untimed_until(lhs: Signal<f64>, rhs: Signal<f64>) -> ArgusResult<Signal<f64>> {
    let sync_points = lhs.sync_with_intersection::<Linear>(&rhs).unwrap();
    let mut ret_samples = Vec::with_capacity(sync_points.len());
    let expected_len = sync_points.len();

    let mut next = f64::NEG_INFINITY;

    for (i, t) in sync_points.into_iter().enumerate().rev() {
        let v1 = lhs.interpolate_at::<Linear>(t).unwrap();
        let v2 = rhs.interpolate_at::<Linear>(t).unwrap();

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

    use argus_core::expr::ExprBuilder;
    use argus_core::signals::AnySignal;
    use itertools::assert_equal;

    use super::*;

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

        let robustness = QuantitativeSemantics::eval_num_expr::<f64>(&spec, &trace).unwrap();

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

            let rob = QuantitativeSemantics::eval_num_expr::<f64>(&spec, &trace).unwrap();
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

            let rob = QuantitativeSemantics::eval_num_expr::<f64>(&spec, &trace).unwrap();
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

        let rob = QuantitativeSemantics::eval(&spec, &trace).unwrap();
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
            let rob = QuantitativeSemantics::eval(&spec, &trace).unwrap();
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
            let rob = QuantitativeSemantics::eval(&spec, &trace).unwrap();

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
            let rob = QuantitativeSemantics::eval(&spec, &trace).unwrap();

            let expected = Signal::from_iter(vec![(Duration::from_secs(4), 3), (Duration::from_secs(6), 3)])
                .num_cast::<f64>()
                .unwrap();

            assert_equal(&rob, &expected);
        }
    }
}
