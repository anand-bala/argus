//! Helper traits for Argus signals.

use std::any::Any;
use std::cmp::Ordering;
use std::time::Duration;

use paste::paste;

use super::utils::Neighborhood;
use super::{Sample, Signal};

/// Trait implemented by interpolation strategies
pub trait InterpolationMethod<T> {
    /// Compute the interpolation of two samples at `time`.
    ///
    /// Returns `None` if it isn't possible to interpolate at the given time using the
    /// given samples.
    fn at(a: &Sample<T>, b: &Sample<T>, time: Duration) -> Option<T>;

    /// Given two signals with two sample points each, find the intersection of the two
    /// lines.
    fn find_intersection(a: &Neighborhood<T>, b: &Neighborhood<T>) -> Option<Sample<T>>;
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
            fn [<signal_ $cmp>]<I>(&self, other: &Rhs) -> Option<Signal<bool>>
            where
                I: InterpolationMethod<T>
            {
                self.signal_cmp::<_, I>(other, |ord| ord.[<is_ $cmp>]())
            }
        }
    };
}

/// A time-wise partial ordering defined for signals
pub trait SignalPartialOrd<T, Rhs = Self> {
    /// Compare two signals within each of their domains (using [`PartialOrd`]) and
    /// apply the given function `op` to the ordering to create a signal.
    ///
    /// This function returns `None` if the comparison isn't possible, namely, when
    /// either of the signals are empty.
    fn signal_cmp<F, I>(&self, other: &Rhs, op: F) -> Option<Signal<bool>>
    where
        F: Fn(Ordering) -> bool,
        I: InterpolationMethod<T>;

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

/// Trait for computing the absolute value of the samples in a signal
pub trait SignalAbs {
    /// Compute the absolute value of the given signal
    fn abs(self) -> Self;
}
