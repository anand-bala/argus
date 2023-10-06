//! # `argus-core`
//!
//! This crate provides some of the core functionality or interfaces for the other Argus
//! components. Mainly, the crate provides:
//!
//! 1. Expression tree nodes for defining temporal logic specifications (see [`expr`]).
//! 2. Different signal types for generating traces of data (see [`signals`]).
//! 3. A list of possible errors any component in Argus can generate (see
//!    [`enum@Error`]).

pub mod expr;
pub mod signals;

pub use expr::*;
pub use signals::Signal;
