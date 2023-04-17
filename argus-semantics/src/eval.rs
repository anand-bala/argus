use argus_core::prelude::*;
use argus_core::signals::traits::{SignalAbs, TrySignalCast};
use num_traits::{Num, NumCast};

use crate::Trace;

pub fn eval_num_expr<T>(root: &NumExpr, trace: &impl Trace) -> ArgusResult<Signal<T>>
where
    T: Num + NumCast,
    Signal<i64>: TrySignalCast<Signal<T>>,
    Signal<u64>: TrySignalCast<Signal<T>>,
    Signal<f64>: TrySignalCast<Signal<T>>,
    for<'a> &'a Signal<T>: std::ops::Neg<Output = Signal<T>>,
    for<'a> &'a Signal<T>: std::ops::Add<&'a Signal<T>, Output = Signal<T>>,
    for<'a> &'a Signal<T>: std::ops::Sub<&'a Signal<T>, Output = Signal<T>>,
    for<'a> &'a Signal<T>: std::ops::Mul<&'a Signal<T>, Output = Signal<T>>,
    for<'a> &'a Signal<T>: std::ops::Div<&'a Signal<T>, Output = Signal<T>>,
    Signal<T>: SignalAbs,
{
    match root {
        NumExpr::IntLit(val) => Signal::constant(*val).try_cast(),
        NumExpr::UIntLit(val) => Signal::constant(*val).try_cast(),
        NumExpr::FloatLit(val) => Signal::constant(*val).try_cast(),
        NumExpr::IntVar { name } => trace.get::<i64>(name.as_str()).unwrap().try_cast(),
        NumExpr::UIntVar { name } => trace.get::<u64>(name.as_str()).unwrap().try_cast(),
        NumExpr::FloatVar { name } => trace.get::<f64>(name.as_str()).unwrap().try_cast(),
        NumExpr::Neg { arg } => eval_num_expr(arg, trace).map(|sig| -&sig),
        NumExpr::Add { args } => {
            let mut ret: Signal<T> = Signal::<T>::zero();
            for arg in args.iter() {
                let arg = eval_num_expr(arg, trace)?;
                ret = &ret + &arg;
            }
            Ok(ret)
        }
        NumExpr::Sub { lhs, rhs } => {
            let lhs = eval_num_expr(lhs, trace)?;
            let rhs = eval_num_expr(rhs, trace)?;
            Ok(&lhs - &rhs)
        }
        NumExpr::Mul { args } => {
            let mut ret: Signal<T> = Signal::<T>::one();
            for arg in args.iter() {
                let arg = eval_num_expr(arg, trace)?;
                ret = &ret * &arg;
            }
            Ok(ret)
        }
        NumExpr::Div { dividend, divisor } => {
            let dividend = eval_num_expr(dividend, trace)?;
            let divisor = eval_num_expr(divisor, trace)?;
            Ok(&dividend / &divisor)
        }
        NumExpr::Abs { arg } => {
            let arg = eval_num_expr(arg, trace)?;
            Ok(arg.abs())
        }
    }
}
