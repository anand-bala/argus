//! Helper traits for Argus signals.

use std::any::Any;
use std::cmp::Ordering;
use std::time::Duration;

use paste::paste;

use super::utils::Neighborhood;
use super::{Sample, Signal};
use crate::ArgusResult;

/// Trait for values that are linear interpolatable
pub trait LinearInterpolatable {
    /// Compute the linear interpolation of two samples at `time`
    ///
    /// This should assume that the `time` value is between the sample times of `a` and
    /// `b`. This should be enforced as an assertion.
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized;

    /// Given two signals with two sample points each, find the intersection of the two
    /// lines.
    fn find_intersection(a: &Neighborhood<Self>, b: &Neighborhood<Self>) -> Sample<Self>
    where
        Self: Sized;
}

impl LinearInterpolatable for bool {
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized,
    {
        assert!(a.time < time && time < b.time);
        // We can't linear interpolate a boolean, so we return the previous.
        a.value
    }

    fn find_intersection(a: &Neighborhood<Self>, b: &Neighborhood<Self>) -> Sample<Self>
    where
        Self: Sized,
    {
        let Sample { time: ta1, value: ya1 } = a.first.unwrap();
        let Sample { time: ta2, value: ya2 } = a.second.unwrap();
        let Sample { time: tb1, value: yb1 } = b.first.unwrap();
        let Sample { time: tb2, value: yb2 } = b.second.unwrap();

        let left_cmp = ya1.cmp(&yb1);
        let right_cmp = ya2.cmp(&yb2);

        if left_cmp.is_eq() {
            // They already intersect, so we return the inner time-point
            if ta1 < tb1 {
                Sample { time: tb1, value: yb1 }
            } else {
                Sample { time: ta1, value: ya1 }
            }
        } else if right_cmp.is_eq() {
            // They intersect at the end, so we return the outer time-point, as that is
            // when they become equal.
            if ta2 < tb2 {
                Sample { time: tb2, value: yb2 }
            } else {
                Sample { time: ta2, value: ya2 }
            }
        } else {
            // The switched, so the one that switched earlier will intersect with the
            // other.
            // So, we find the one that has a lower time point, i.e., the inner one.
            if ta2 < tb2 {
                Sample { time: ta2, value: ya2 }
            } else {
                Sample { time: tb2, value: yb2 }
            }
        }
    }
}

macro_rules! interpolate_for_num {
    ($ty:ty) => {
        impl LinearInterpolatable for $ty {
            fn interpolate_at(first: &Sample<Self>, second: &Sample<Self>, time: Duration) -> Self
            where
                Self: Sized,
            {
                use num_traits::cast;
                // We will need to cast the samples to f64 values (along with the time
                // window) to be able to interpolate correctly.
                // TODO(anand): Verify this works.
                let t1 = first.time.as_secs_f64();
                let t2 = second.time.as_secs_f64();
                let at = time.as_secs_f64();
                assert!((t1..=t2).contains(&at));

                // We need to do stable linear interpolation
                // https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2019/p0811r3.html
                let a: f64 = cast(first.value).unwrap();
                let b: f64 = cast(second.value).unwrap();

                // Set t to a value in [0, 1]
                let t = (at - t1) / (t2 - t1);
                assert!((0.0..=1.0).contains(&t));

                let val = if (a <= 0.0 && b >= 0.0) || (a >= 0.0 && b <= 0.0) {
                    t * b + (1.0 - t) * a
                } else if t == 1.0 {
                    b
                } else {
                    a + t * (b - a)
                };

                cast(val).unwrap()
            }

            fn find_intersection(a: &Neighborhood<Self>, b: &Neighborhood<Self>) -> Sample<Self>
            where
                Self: Sized,
            {
                // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line
                use num_traits::cast;

                let Sample { time: t1, value: y1 } = a.first.unwrap();
                let Sample { time: t2, value: y2 } = a.second.unwrap();
                let Sample { time: t3, value: y3 } = b.first.unwrap();
                let Sample { time: t4, value: y4 } = b.second.unwrap();

                let t1 = t1.as_secs_f64();
                let t2 = t2.as_secs_f64();
                let t3 = t3.as_secs_f64();
                let t4 = t4.as_secs_f64();

                let y1: f64 = cast(y1).unwrap();
                let y2: f64 = cast(y2).unwrap();
                let y3: f64 = cast(y3).unwrap();
                let y4: f64 = cast(y4).unwrap();

                let denom = ((t1 - t2) * (y3 - y4)) - ((y1 - y2) * (t3 - t4));

                let t_top = (((t1 * y2) - (y1 * t2)) * (t3 - t4)) - ((t1 - t2) * (t3 * y4 - y3 * t4));
                let y_top = (((t1 * y2) - (y1 * t2)) * (y3 - y4)) - ((y1 - y2) * (t3 * y4 - y3 * t4));

                let t = Duration::from_secs_f64(t_top / denom);
                let y: Self = cast(y_top / denom).unwrap();
                Sample { time: t, value: y }
            }
        }
    };
}

interpolate_for_num!(i8);
interpolate_for_num!(i16);
interpolate_for_num!(i32);
interpolate_for_num!(i64);
interpolate_for_num!(u8);
interpolate_for_num!(u16);
interpolate_for_num!(u32);
interpolate_for_num!(u64);
interpolate_for_num!(f32);
interpolate_for_num!(f64);

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
