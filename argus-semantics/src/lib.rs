//! Argus offline semantics
//!
//! In this crate, we are predominantly concerned with the monitoring of _offline system
//! traces_, i.e., a collection of signals that have been extracted from observing and
//! sampling from some system.
use argus_core::expr::BoolExpr;
use argus_core::signals::AnySignal;
use argus_core::ArgusResult;

pub mod eval;
pub mod semantics;
pub mod utils;

pub use semantics::boolean::BooleanSemantics;
pub use semantics::quantitative::QuantitativeSemantics;

/// A trace is a collection of signals
///
/// # Example
///
/// An example of a `Trace` may be:
///
/// ```rust
/// # use std::collections::HashMap;
/// # use argus_core::signals::{ConstantSignal, AnySignal};
/// # use argus_semantics::Trace;
///
/// struct MyTrace(HashMap<String, AnySignal>);
///
/// impl Trace for MyTrace {
///     fn signal_names(&self) -> Vec<&str> {
///         self.0.keys().map(|k| k.as_str()).collect()
///     }
///
///     fn get(&self, name: &str) -> Option<&AnySignal> {
///         self.0.get(name)
///     }
/// }
///
/// let trace = MyTrace(HashMap::from([
///     ("x".to_string(), ConstantSignal::new(true).into()),
///     ("y".to_string(), ConstantSignal::new(2.0).into()),
/// ]));
/// let names = trace.signal_names();
///
/// assert!(names == &["x", "y"] || names == &["y", "x"]);
/// assert!(matches!(trace.get("x"), Some(AnySignal::ConstBool(_))));
/// assert!(matches!(trace.get("y"), Some(AnySignal::ConstFloat(_))));
/// ```
pub trait Trace {
    /// Get the list of signal names contained within the trace.
    fn signal_names(&self) -> Vec<&str>;

    /// Query a signal using its name
    fn get(&self, name: &str) -> Option<&AnySignal>;
}

/// General interface for defining semantics for the [`argus-core`](argus_core) logic.
pub trait Semantics {
    /// The output of applying the given semantics to an expression and trace.
    type Output;
    /// Any additional possible context that can be passed to the semantics evaluator.
    type Context;

    fn eval(expr: &BoolExpr, trace: &impl Trace, ctx: Self::Context) -> ArgusResult<Self::Output>;
}
