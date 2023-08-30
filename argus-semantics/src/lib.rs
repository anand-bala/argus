//! Argus offline semantics
//!
//! In this crate, we are predominantly concerned with the monitoring of _offline system
//! traces_, i.e., a collection of signals that have been extracted from observing and
//! sampling from some system.

pub mod semantics;
pub mod traits;
pub mod utils;

pub use semantics::{BooleanSemantics, QuantitativeSemantics};
pub use traits::Trace;
