//! Traits to define semantics for temporal logic specifications

use argus_core::prelude::*;

/// A trace is a collection of signals
///
/// # Example
///
/// An example of a `Trace` may be:
///
/// ```rust
/// use argus_core::signals::{Signal, AnySignal};
/// use argus_semantics::Trace;
///
/// struct MyTrace {
///     x: Signal<bool>,
///     y: Signal<i64>,
/// }
///
/// impl Trace for MyTrace {
///     fn signal_names(&self) -> Vec<&str> {
///         vec!["x", "y"]
///     }
///
///     fn get<T: 'static>(&self, name: &str) -> Option<&Signal<T>> {
///         let sig: &dyn AnySignal = match name {
///             "x" => &self.x,
///             "y" => &self.y,
///             _ => return None,
///         };
///         sig.as_any().downcast_ref::<Signal<T>>()
///     }
/// }
///
/// let trace = MyTrace {
///     x: Signal::constant(true),
///     y: Signal::constant(2),
/// };
/// let names = trace.signal_names();
///
/// assert!(names == &["x", "y"] || names == &["y", "x"]);
/// assert!(matches!(trace.get::<bool>("x"), Some(Signal::Constant { value: true })));
/// assert!(matches!(trace.get::<i64>("y"), Some(Signal::Constant { value: 2 })));
/// ```
pub trait Trace {
    /// Get the list of signal names contained within the trace.
    fn signal_names(&self) -> Vec<&str>;

    /// Query a signal using its name
    fn get<T: 'static>(&self, name: &str) -> Option<&Signal<T>>;
}
