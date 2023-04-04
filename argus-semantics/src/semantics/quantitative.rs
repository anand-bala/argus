use argus_core::expr::BoolExpr;
use argus_core::prelude::*;
use argus_core::signals::traits::{BaseSignal, SignalAbs, SignalMinMax, SignalNumCast};

use crate::eval::NumExprEval;
use crate::{Semantics, Trace};

macro_rules! num_signal_binop_impl {
    ($lhs:ident, $rhs:ident, $op:ident, [$( $type:ident ), *]) => {
        paste::paste!{
            {
            use argus_core::prelude::*;
            use argus_core::ArgusError;
            use AnySignal::*;
            match ($lhs, $rhs) {
                $(
                    ([<$type >](lhs), [<  $type >](rhs)) => Ok(AnySignal::from($op(&lhs, &rhs))),
                    ([<$type >](lhs), [< Const $type >](rhs)) => Ok(AnySignal::from($op(&lhs, &rhs))),
                    ([<Const $type >](lhs), [<  $type >](rhs)) => Ok(AnySignal::from($op(&lhs, &rhs))),
                    ([<Const $type >](lhs), [< Const $type >](rhs)) => Ok(AnySignal::from($op(&lhs, &rhs))),
                )*
                _ => Err(ArgusError::InvalidOperation),
                }
            }
        }
    };
}

fn less_than<T, Sig1, Sig2, Ret>(lhs: &Sig1, rhs: &Sig2) -> Ret
where
    Sig1: SignalNumCast + BaseSignal<Value = T>,
    Sig2: SignalNumCast + BaseSignal<Value = T>,
    Ret: BaseSignal<Value = f64>,
    for<'a> &'a <Sig2 as SignalNumCast>::Output<f64>:
        std::ops::Sub<&'a <Sig1 as SignalNumCast>::Output<f64>, Output = Ret>,
{
    let lhs = lhs.to_f64().unwrap();
    let rhs = rhs.to_f64().unwrap();
    &rhs - &lhs
}

fn greater_than<T, Sig1, Sig2, Ret>(lhs: &Sig1, rhs: &Sig2) -> Ret
where
    Sig1: SignalNumCast + BaseSignal<Value = T>,
    Sig2: SignalNumCast + BaseSignal<Value = T>,
    Ret: BaseSignal<Value = f64>,
    for<'a> &'a <Sig1 as SignalNumCast>::Output<f64>:
        std::ops::Sub<&'a <Sig2 as SignalNumCast>::Output<f64>, Output = Ret>,
{
    let lhs = lhs.to_f64().unwrap();
    let rhs = rhs.to_f64().unwrap();
    &lhs - &rhs
}

fn equal_to<T, Sig1, Sig2, Ret>(lhs: &Sig1, rhs: &Sig2) -> Ret
where
    Sig1: SignalNumCast + BaseSignal<Value = T>,
    Sig2: SignalNumCast + BaseSignal<Value = T>,
    Ret: BaseSignal<Value = f64> + SignalAbs,
    for<'a> &'a <Sig1 as SignalNumCast>::Output<f64>:
        std::ops::Sub<&'a <Sig2 as SignalNumCast>::Output<f64>, Output = Ret>,
{
    let lhs = lhs.to_f64().unwrap();
    let rhs = rhs.to_f64().unwrap();
    (&lhs - &rhs).abs()
}

fn not_equal_to<T, Sig1, Sig2, Ret>(lhs: &Sig1, rhs: &Sig2) -> Ret
where
    Sig1: SignalNumCast + BaseSignal<Value = T>,
    Sig2: SignalNumCast + BaseSignal<Value = T>,
    Ret: BaseSignal<Value = f64> + SignalAbs,
    for<'a> &'a Ret: core::ops::Neg<Output = Ret>,
    for<'a> &'a <Sig1 as SignalNumCast>::Output<f64>:
        std::ops::Sub<&'a <Sig2 as SignalNumCast>::Output<f64>, Output = Ret>,
{
    let lhs = lhs.to_f64().unwrap();
    let rhs = rhs.to_f64().unwrap();
    -&((&lhs - &rhs).abs())
}

macro_rules! signal_bool_op_impl {
    // Unary bool opeartions
    (! $signal:ident) => {{
        use argus_core::prelude::*;
        use AnySignal::*;
        match $signal {
            Float(sig) => Ok(AnySignal::from(-(&sig))),
            ConstFloat(sig) => Ok(AnySignal::from(-(&sig))),
            _ => unreachable!("no other signal is expected in quantitative semantics"),
        }
    }};

    ($op:ident, $lhs:ident, $rhs:ident) => {
        paste::paste! {
            {
            use argus_core::prelude::*;
            use AnySignal::*;
            match ($lhs, $rhs) {
                (Float(lhs), Float(rhs)) => AnySignal::from(lhs.$op(&rhs)),
                (Float(lhs), ConstFloat(rhs)) => AnySignal::from(lhs.$op(&rhs)),
                (ConstFloat(lhs), Float(rhs)) => AnySignal::from(lhs.$op(&rhs)),
                (ConstFloat(lhs), ConstFloat(rhs)) => AnySignal::from(lhs.$op(&rhs)),
                _ => panic!("mismatched argument types for {} operation", stringify!($op)),
                }
            }
        }
    };
}

fn bool_to_f64_sig(sig: &Signal<bool>) -> Signal<f64> {
    sig.iter()
        .map(|(&t, &v)| if v { (t, f64::INFINITY) } else { (t, f64::NEG_INFINITY) })
        .collect()
}

fn bool_to_f64_const_sig(sig: &ConstantSignal<bool>) -> ConstantSignal<f64> {
    if sig.value {
        ConstantSignal::new(f64::INFINITY)
    } else {
        ConstantSignal::new(f64::NEG_INFINITY)
    }
}

fn top_or_bot_sig(val: bool) -> ConstantSignal<f64> {
    if val {
        ConstantSignal::new(f64::INFINITY)
    } else {
        ConstantSignal::new(f64::NEG_INFINITY)
    }
}

/// Quantitative semantics for Argus expressions
pub struct QuantitativeSemantics;

impl Semantics for QuantitativeSemantics {
    type Output = AnySignal;
    type Context = ();

    fn eval(expr: &BoolExpr, trace: &impl Trace, ctx: Self::Context) -> ArgusResult<Self::Output> {
        match expr {
            BoolExpr::BoolLit(val) => Ok(top_or_bot_sig(*val).into()),
            BoolExpr::BoolVar { name } => {
                let sig = trace.get(name.as_str()).ok_or(ArgusError::SignalNotPresent)?;
                match sig {
                    AnySignal::ConstBool(bool_sig) => Ok(bool_to_f64_const_sig(bool_sig).into()),
                    AnySignal::Bool(sig) => Ok(bool_to_f64_sig(sig).into()),
                    _ => Err(ArgusError::InvalidSignalType),
                }
            }
            BoolExpr::Cmp { op, lhs, rhs } => {
                use argus_core::expr::Ordering::*;
                let lhs = NumExprEval::eval(lhs, trace);
                let rhs = NumExprEval::eval(rhs, trace);

                match op {
                    Eq => num_signal_binop_impl!(lhs, rhs, equal_to, [Int, UInt, Float]),
                    NotEq => num_signal_binop_impl!(lhs, rhs, not_equal_to, [Int, UInt, Float]),
                    Less { strict: _ } => num_signal_binop_impl!(lhs, rhs, less_than, [Int, UInt, Float]),
                    Greater { strict: _ } => num_signal_binop_impl!(lhs, rhs, greater_than, [Int, UInt, Float]),
                }
            }
            BoolExpr::Not { arg } => {
                let arg = Self::eval(arg, trace, ctx)?;
                signal_bool_op_impl!(!arg)
            }
            BoolExpr::And { args } => {
                assert!(args.len() >= 2);
                let args = args
                    .iter()
                    .map(|arg| Self::eval(arg, trace, ctx))
                    .collect::<ArgusResult<Vec<_>>>()?;
                args.into_iter()
                    .reduce(|lhs, rhs| signal_bool_op_impl!(min, lhs, rhs))
                    .ok_or(ArgusError::InvalidOperation)
            }
            BoolExpr::Or { args } => {
                assert!(args.len() >= 2);
                let args = args
                    .iter()
                    .map(|arg| Self::eval(arg, trace, ctx))
                    .collect::<ArgusResult<Vec<_>>>()?;
                args.into_iter()
                    .reduce(|lhs, rhs| signal_bool_op_impl!(max, lhs, rhs))
                    .ok_or(ArgusError::InvalidOperation)
            }
        }
    }
}
