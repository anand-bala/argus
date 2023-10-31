use super::Trace;
use crate::core::expr::*;
use crate::core::signals::{InterpolationMethod, SignalPartialOrd};
use crate::semantics::QuantitativeSemantics;
use crate::{ArgusResult, Signal};

/// Boolean semantics for Signal Temporal Logic expressionsd define by an [`Expr`].
pub struct BooleanSemantics;

impl BooleanSemantics {
    /// Evaluates a [Boolean expression](BoolExpr) given a [`Trace`].
    pub fn eval<I>(expr: &BoolExpr, trace: &impl Trace) -> ArgusResult<Signal<bool>>
    where
        I: InterpolationMethod<f64>,
    {
        let rob = QuantitativeSemantics::eval::<I>(expr, trace)?;
        Ok(rob.signal_ge::<I>(&Signal::zero()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::collections::HashMap;

    use itertools::assert_equal;

    use super::*;
    use crate::core::expr::ExprBuilder;
    use crate::core::signals::interpolation::Linear;
    use crate::core::signals::AnySignal;
    use crate::signals::interpolation::Constant;

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

        let rob = BooleanSemantics::eval::<Linear>(&spec, &trace).unwrap();
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
            let rob = run_test_float_time::<Linear, _, _>(
                vec![(
                    "a".to_owned(),
                    vec![((0.0), 2.5), ((0.7), 4.0), ((1.3), -1.0), ((2.1), 1.7)],
                )],
                &spec,
            );

            let Signal::Sampled { values, time_points: _ } = rob else {
                panic!("boolean semantics should remain sampled");
            };
            assert!(values.into_iter().all(|v| v));
        }
        {
            let rob = run_test_float_time::<Linear, _, _>(
                vec![(
                    "a".to_owned(),
                    (vec![((0.0), 2.5), ((0.7), 4.0), ((1.3), 1.7), ((1.4), 0.0), ((2.1), -2.0)]),
                )],
                &spec,
            );
            println!("{:#?}", rob);

            let Signal::Sampled { values, time_points: _ } = rob else {
                panic!("boolean semantics should remain sampled");
            };
            assert!(values[..values.len() - 1].iter().all(|&v| v));
            assert!(!values[values.len() - 1]);
        }
    }

    #[test]
    fn smoketest_1() {
        let Expr::Bool(spec) = crate::parse_str("G(a -> F[0,2] b)").unwrap() else {
            panic!("should be bool expr")
        };
        let rob = run_test_float_time::<Constant, _, bool>(
            vec![
                ("a".to_owned(), vec![(1.0, false), (2.0, false), (3.0, false)]),
                ("b".to_owned(), vec![(1.0, false), (2.0, true), (3.0, false)]),
            ],
            &spec,
        );

        let Signal::Sampled { values, time_points: _ } = rob else {
            panic!("boolean semantics should remain sampled");
        };
        assert!(values.into_iter().all(|v| v));
    }

    #[test]
    fn smoketest_2() {
        let Expr::Bool(spec) = crate::parse_str("a").unwrap() else {
            panic!("should be bool expr")
        };
        let rob = run_test_float_time::<Linear, _, bool>(
            vec![
                ("a".to_owned(), vec![(0.0, false), (196.864, true), (12709.888, true)]),
                ("b".to_owned(), vec![(0.0, false), (196.864, false), (12709.888, false)]),
            ],
            &spec,
        );

        let Signal::Sampled { values, time_points: _ } = rob else {
            panic!("boolean semantics should remain sampled");
        };
        assert_eq!(values, vec![false, true, true]);
    }

    #[test]
    fn smoketest_3() {
        //  {
        //      sample_lists=[
        //          [(0.0, False), (0.001, False), (2.001, False)],
        //          [(0.0, False), (0.001, False), (2.001, False)]
        //      ],
        //      spec='F[0,2] a',
        //      interpolation_method='constant',
        //  }
        let Expr::Bool(spec) = crate::parse_str("F[0,2] a").unwrap() else {
            panic!("should be bool expr")
        };
        let rob = run_test_float_time::<Linear, _, bool>(
            vec![
                ("a".to_owned(), vec![(0.0, false), (0.001, false), (2.002, false)]),
                ("b".to_owned(), vec![(0.0, false), (0.001, false), (2.002, false)]),
            ],
            &spec,
        );

        let Signal::Sampled { values, time_points: _ } = rob else {
            panic!("boolean semantics should remain sampled");
        };
        assert!(values.into_iter().all(|v| !v));
    }

    #[test]
    fn smoketest_4() {
        //  {
        //      sample_lists = [
        //          [(0.0, False), (0.001, False), (4.002, False)],
        //          [(0.0, False), (0.001, False), (4.002, False)]
        //      ],
        //      spec = 'F[0,2] a',
        //      interpolation_method = 'constant'
        //  }
        let Expr::Bool(spec) = crate::parse_str("F[0,2] a").unwrap() else {
            panic!("should be bool expr")
        };
        let rob = run_test_float_time::<Linear, _, bool>(
            vec![
                ("a".to_owned(), vec![(0.0, false), (0.001, false), (4.002, false)]),
                ("b".to_owned(), vec![(0.0, false), (0.001, false), (4.002, false)]),
            ],
            &spec,
        );

        let Signal::Sampled { values, time_points: _ } = rob else {
            panic!("boolean semantics should remain sampled");
        };
        assert!(values.into_iter().all(|v| !v));
    }

    fn run_test_float_time<Interp, I, T>(signals: I, spec: &BoolExpr) -> Signal<bool>
    where
        I: IntoIterator<Item = (String, Vec<(f64, T)>)>,
        T: Copy + core::fmt::Debug + 'static,
        Interp: InterpolationMethod<f64>,
    {
        let signals: HashMap<String, Box<dyn AnySignal>> = signals
            .into_iter()
            .map(|(name, samples)| {
                (
                    name,
                    Box::new(
                        samples
                            .into_iter()
                            .map(|(t, v)| (Duration::from_secs_f64(t), v))
                            .collect::<Signal<T>>(),
                    ) as Box<dyn AnySignal>,
                )
            })
            .collect();

        let trace = MyTrace { signals };
        BooleanSemantics::eval::<Interp>(spec, &trace).unwrap()
    }
}
