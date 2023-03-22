use std::ops::RangeBounds;
use std::time::Duration;

use num_traits::Num;

use super::{InterpolationMethod, Sample};
use crate::ArgusResult;

/// A general Signal trait
pub trait BaseSignal {
    /// Type of the values contained in the signal.
    ///
    /// For example, a signal that implements `BaseSignal<Value = f64, ...>` contains a
    /// sequence of timestamped `f64` values.
    type Value;

    /// A type that implements [`RangeBounds`] to determine the duration bounds of the
    /// signal.
    ///
    /// In practice, this should only be either [`RangeFull`](core::ops::RangeFull)
    /// (returned by constant signals) or [`Range`](core::ops::Range) (returned by
    /// sampled signals).
    type Bounds: RangeBounds<Duration>;

    /// Get the value of the signal at the given time point
    ///
    /// If there exists a sample at the given time point then `Some(value)` is returned.
    /// Otherwise, `None` is returned. If the goal is to interpolate the value at the
    /// a given time, see [`interpolate_at`](Self::interpolate_at).
    fn at(&self, time: Duration) -> Option<&Self::Value>;

    /// Interpolate the value of the signal at the given time point
    ///
    /// If there exists a sample at the given time point then `Some(value)` is returned
    /// with the value of the signal at the point. Otherwise, a the
    /// [`InterpolationMethod`] is used to compute the value. If the given interpolation
    /// method cannot be used at the given time (for example, if we use
    /// [`InterpolationMethod::Linear`] and the `time` point is outside the signal
    /// domain), then a `None` is returned.
    fn interpolate_at(&self, time: Duration, interp: InterpolationMethod) -> Option<Self::Value>
    where
        Self::Value: Copy + LinearInterpolatable;

    /// Get the bounds for the signal
    fn bounds(&self) -> Self::Bounds;

    /// Push a new sample to the signal at the given time point
    ///
    /// The method should enforce the invariant that the time points of the signal must
    /// have strictly monotonic increasing values, otherwise it returns an error without
    /// adding the sample point.
    ///
    /// The result contains `true` if the sample was successfully added. For example,
    /// pusing a value to a [constant signal](crate::signals::constant) will be a no-op
    /// and return `false`.
    fn push(&mut self, time: Duration, value: Self::Value) -> ArgusResult<bool>;

    /// Check if the signal is empty
    fn is_empty(&self) -> bool {
        use core::ops::Bound::*;
        let bounds = self.bounds();
        match (bounds.start_bound(), bounds.end_bound()) {
            (Included(start), Included(end)) => start > end,
            (Included(start), Excluded(end)) | (Excluded(start), Included(end)) | (Excluded(start), Excluded(end)) => {
                start >= end
            }

            (Unbounded, Unbounded) => false,
            bound => unreachable!("Argus doesn't support signals with bound {:?}", bound),
        }
    }

    /// Get the time at which the given signal starts.
    fn start_time(&self) -> core::ops::Bound<Duration> {
        self.bounds().start_bound().cloned()
    }

    /// Get the time at which the given signal ends.
    fn end_time(&self) -> core::ops::Bound<Duration> {
        self.bounds().end_bound().cloned()
    }
}

/// A Boolean signal
pub trait BaseBooleanSignal: BaseSignal {}

/// A numeric signal
pub trait BaseNumericSignal: BaseSignal {
    type Value: Num;
}

/// Trait for values that are linear interpolatable
pub trait LinearInterpolatable {
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized;
}

impl LinearInterpolatable for bool {
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized,
    {
        use InterpolationMethod::Nearest;
        assert!(a.time < time && time < b.time);
        // We can't linear interpolate a boolean, so we return the nearest.
        Nearest.at(time, &Some(*a), &Some(*b)).unwrap()
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