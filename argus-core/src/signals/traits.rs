//! Helper traits for Argus signals.

use std::any::Any;
use std::cmp::Ordering;
use std::time::Duration;

use paste::paste;

use super::utils::Neighborhood;
use super::{Sample, Signal};
use crate::ArgusResult;

/// Trait implemented by interpolation strategies
pub trait InterpolationMethod<T> {
    /// Compute the interpolation of two samples at `time`.
    ///
    /// Returns `None` if it isn't possible to interpolate at the given time using the
    /// given samples.
    fn at(a: &Sample<T>, b: &Sample<T>, time: Duration) -> Option<T>;
}

/// Trait implemented by interpolation strategies that allow finding the intersection of
/// two signal segments defined by start and end samples (see [`Neighborhood`]).
pub trait FindIntersectionMethod<T>: InterpolationMethod<T> {
    /// Given two signals with two sample points each, find the intersection of the two
    /// lines.
    fn find_intersection(a: &Neighborhood<T>, b: &Neighborhood<T>) -> Sample<T>;
}

/// Simple trait to be used as a trait object for [`Signal<T>`] types.
///
/// This is mainly for external libraries to use for trait objects and downcasting to
/// concrete [`Signal`] types.
pub trait AnySignal {
    /// Convenience method to upcast a signal to [`std::any::Any`] for later downcasting
    /// to a concrete [`Signal`] type.
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AnySignal for Signal<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

macro_rules! impl_signal_cmp {
    ($cmp:ident) => {
        paste! {
            /// Compute the time-wise comparison of two signals
            fn [<signal_ $cmp>](&self, other: &Rhs) -> Option<Signal<bool>> {
                self.signal_cmp(other, |ord| ord.[<is_ $cmp>]())
            }
        }
    };
}

/// A time-wise partial ordering defined for signals
pub trait SignalPartialOrd<Rhs = Self> {
    /// Compare two signals within each of their domains (using [`PartialOrd`]) and
    /// apply the given function `op` to the ordering to create a signal.
    ///
    /// This function returns `None` if the comparison isn't possible, namely, when
    /// either of the signals are empty.
    fn signal_cmp<F>(&self, other: &Rhs, op: F) -> Option<Signal<bool>>
    where
        F: Fn(Ordering) -> bool;

    impl_signal_cmp!(lt);
    impl_signal_cmp!(le);
    impl_signal_cmp!(gt);
    impl_signal_cmp!(ge);
    impl_signal_cmp!(eq);
    impl_signal_cmp!(ne);
}

/// Time-wise min-max of signal types
pub trait SignalMinMax<Rhs = Self> {
    /// The output type of the signal after computing the min/max
    type Output;

    /// Compute the time-wise min of two signals
    fn min(&self, rhs: &Rhs) -> Self::Output;

    /// Compute the time-wise max of two signals
    fn max(&self, rhs: &Rhs) -> Self::Output;
}

/// Trait for converting between signal types
pub trait SignalNumCast {
    /// Try to convert the signal values/samples to `i8`
    fn to_i8(&self) -> Option<Signal<i8>>;
    /// Try to convert the signal values/samples to `i16`
    fn to_i16(&self) -> Option<Signal<i16>>;
    /// Try to convert the signal values/samples to `i32`
    fn to_i32(&self) -> Option<Signal<i32>>;
    /// Try to convert the signal values/samples to `i64`
    fn to_i64(&self) -> Option<Signal<i64>>;
    /// Try to convert the signal values/samples to `u8`
    fn to_u8(&self) -> Option<Signal<u8>>;
    /// Try to convert the signal values/samples to `u16`
    fn to_u16(&self) -> Option<Signal<u16>>;
    /// Try to convert the signal values/samples to `u32`
    fn to_u32(&self) -> Option<Signal<u32>>;
    /// Try to convert the signal values/samples to `u64`
    fn to_u64(&self) -> Option<Signal<u64>>;
    /// Try to convert the signal values/samples to `f32`
    fn to_f32(&self) -> Option<Signal<f32>>;
    /// Try to convert the signal values/samples to `f64`
    fn to_f64(&self) -> Option<Signal<f64>>;
}

/// Trait to cast signal onto some type
pub trait TrySignalCast<T>: Sized + SignalNumCast {
    /// Try to cast the given signal to another numeric type.
    ///
    /// This returns a [`ArgusError::InvalidCast`](crate::Error::InvalidCast) if
    /// some value in the signal isn't castable to the destination type.
    fn try_cast(&self) -> ArgusResult<T>;
}

/// Trait for computing the absolute value of the samples in a signal
pub trait SignalAbs {
    /// Compute the absolute value of the given signal
    fn abs(&self) -> Self;
}
