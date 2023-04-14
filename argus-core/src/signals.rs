//! Concrete signal types
//!
//! In Argus, there are essentially 2 kinds of signals:
//!
//! 1. [`Signal<T>`] is a variable length signal with finitely many sampled points. This
//!    implies that the signal has a fixed start and end point (both inclusive) and can
//!    be iterated over.
//! 2. [`ConstantSignal<T>`] is a signal that maintains a constant value throughtout
//!    its domain, and thus, do not require interpolation and extrapolation. Moreover,
//!    since they are defined over the entire time domain, they cannot be iterated over.
pub mod bool_ops;
pub mod cast;
pub mod cmp_ops;
pub mod iter;
pub mod num_ops;
pub mod traits;
mod utils;

use std::ops::{Bound, RangeBounds};
use std::time::Duration;

pub use bool_ops::*;
pub use cast::*;
pub use cmp_ops::*;
use itertools::Itertools;
pub use num_ops::*;
use num_traits::NumCast;
use utils::intersect_bounds;

use self::traits::LinearInterpolatable;
use crate::{ArgusResult, Error};

#[derive(Debug, Clone, Copy)]
pub enum InterpolationMethod {
    Linear,
    Nearest,
}

impl InterpolationMethod {
    pub(crate) fn at<T>(self, time: Duration, a: &Option<Sample<T>>, b: &Option<Sample<T>>) -> Option<T>
    where
        T: Copy + LinearInterpolatable,
    {
        use InterpolationMethod::*;
        match (self, a, b) {
            (Nearest, Some(ref a), Some(ref b)) => {
                assert!(a.time < time && time < b.time);
                if (b.time - time) > (time - a.time) {
                    // a is closer to the required time than b
                    Some(a.value)
                } else {
                    // b is closer
                    Some(b.value)
                }
            }
            (Nearest, Some(nearest), None) | (Nearest, None, Some(nearest)) => Some(nearest.value),
            (Linear, Some(a), Some(b)) => Some(T::interpolate_at(a, b, time)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sample<T> {
    pub time: Duration,
    pub value: T,
}

/// A typed Signal
///
/// A Signal can either be empty, constant throughout its domain, or sampled at a
/// finite set of strictly monotonically increasing time points.
#[derive(Default, Clone, Debug)]
pub enum Signal<T> {
    #[default]
    Empty,
    Constant {
        value: T,
    },
    Sampled {
        values: Vec<T>,
        time_points: Vec<Duration>,
    },
}

impl<T> Signal<T> {
    /// Create a new empty signal
    pub fn new() -> Self {
        Self::Empty
    }

    /// Create a new constant signal
    pub fn constant(value: T) -> Self {
        Self::Constant { value }
    }

    /// Create a new empty signal with the specified capacity
    pub fn new_with_capacity(size: usize) -> Self {
        Self::Sampled {
            values: Vec::with_capacity(size),
            time_points: Vec::with_capacity(size),
        }
    }

    /// Get the bounds of the signal.
    ///
    /// Returns `None` if the signal is empty (either [`Signal::Empty`] or
    /// [`Signal::Sampled`] with no samples.
    pub fn bounds(&self) -> Option<(Bound<Duration>, Bound<Duration>)> {
        use core::ops::Bound::*;
        match self {
            Signal::Empty => None,
            Signal::Constant { value: _ } => Some((Unbounded, Unbounded)),
            Signal::Sampled { values: _, time_points } => {
                if time_points.is_empty() {
                    None
                } else {
                    Some((Included(time_points[0]), Included(*time_points.last().unwrap())))
                }
            }
        }
    }

    /// Check if the signal is empty
    pub fn is_empty(&self) -> bool {
        use core::ops::Bound::*;
        let bounds = match self.bounds() {
            Some(b) => b,
            None => return true,
        };
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
    pub fn start_time(&self) -> Option<Bound<Duration>> {
        self.bounds().map(|b| b.0)
    }

    /// Get the time at which the given signal ends.
    pub fn end_time(&self) -> Option<Bound<Duration>> {
        self.bounds().map(|b| b.1)
    }

    /// Push a new sample to the signal at the given time point
    ///
    /// The method enforces the invariant that the time points of the signal must have
    /// strictly monotonic increasing values, otherwise it returns an error without
    /// adding the sample point.
    /// Moreover, it is an error to `push` a value to an [`Empty`](Signal::Empty) or
    /// [`Constant`](Signal::Constant) signal.
    pub fn push(&mut self, time: Duration, value: T) -> ArgusResult<()> {
        match self {
            Signal::Empty | Signal::Constant { value: _ } => Err(Error::InvalidPushToSignal),
            Signal::Sampled { values, time_points } => {
                let last_time = time_points.last();
                match last_time {
                    Some(last_t) if last_t >= &time => Err(Error::NonMonotonicSignal {
                        end_time: *last_t,
                        current_sample: time,
                    }),
                    _ => {
                        time_points.push(time);
                        values.push(value);
                        Ok(())
                    }
                }
            }
        }
    }

    /// Create an iterator over the pairs of time points and values of the signal.
    pub fn iter(&self) -> impl Iterator<Item = (&Duration, &T)> {
        self.into_iter()
    }

    /// Try to create a signal from the input iterator
    ///
    /// Returns an `Err` if the input samples are not in strictly monotonically
    /// increasing order.
    pub fn try_from_iter<I>(iter: I) -> ArgusResult<Self>
    where
        I: IntoIterator<Item = (Duration, T)>,
    {
        let iter = iter.into_iter();
        let mut signal = Signal::new_with_capacity(iter.size_hint().0);
        for (time, value) in iter.into_iter() {
            signal.push(time, value)?;
        }
        Ok(signal)
    }

    /// Get the value of the signal at the given time point
    ///
    /// If there exists a sample at the given time point then `Some(value)` is returned.
    /// Otherwise, `None` is returned. If the goal is to interpolate the value at the
    /// a given time, see [`interpolate_at`](Self::interpolate_at).
    pub fn at(&self, time: Duration) -> Option<&T> {
        match self {
            Signal::Empty => None,
            Signal::Constant { value } => Some(value),
            Signal::Sampled { values, time_points } => {
                assert_eq!(
                    time_points.len(),
                    values.len(),
                    "invariant: number of time points must equal number of samples"
                );
                // if there are no sample points, then there is no sample point (nor neighboring
                // sample points) to return
                if time_points.is_empty() {
                    return None;
                }

                // We will use binary search to find the appropriate index
                match time_points.binary_search(&time) {
                    Ok(idx) => values.get(idx),
                    Err(_) => None,
                }
            }
        }
    }

    /// Interpolate the value of the signal at the given time point
    ///
    /// If there exists a sample at the given time point then `Some(value)` is returned
    /// with the value of the signal at the point. Otherwise, a the
    /// [`InterpolationMethod`] is used to compute the value. If the given interpolation
    /// method cannot be used at the given time (for example, if we use
    /// [`InterpolationMethod::Linear`] and the `time` point is outside the signal
    /// domain), then a `None` is returned.
    pub fn interpolate_at(&self, time: Duration, interp: InterpolationMethod) -> Option<T>
    where
        T: Copy + LinearInterpolatable,
    {
        match self {
            Signal::Empty => None,
            Signal::Constant { value } => Some(*value),
            Signal::Sampled { values, time_points } => {
                assert_eq!(
                    time_points.len(),
                    values.len(),
                    "invariant: number of time points must equal number of samples"
                );
                // if there are no sample points, then there is no sample point (nor neighboring
                // sample points) to return
                if time_points.is_empty() {
                    return None;
                }

                // We will use binary search to find the appropriate index
                let hint_idx = match time_points.binary_search(&time) {
                    Ok(idx) => return values.get(idx).copied(),
                    Err(idx) => idx,
                };

                // We have an hint as to where the sample _should have been_.
                // So, lets check if there is a preceding and/or following sample.
                let (first, second) = if hint_idx == 0 {
                    // Sample appears before the start of the signal
                    // So, let's return just the following sample, which is the first sample
                    // (since we know that the signal is non-empty).
                    let preceding = None;
                    let following = Some(Sample {
                        time: time_points[hint_idx],
                        value: values[hint_idx],
                    });
                    (preceding, following)
                } else if hint_idx == time_points.len() {
                    // Sample appears past the end of the signal
                    // So, let's return just the preceding sample, which is the last sample
                    // (since we know the signal is non-empty)
                    let preceding = Some(Sample {
                        time: time_points[hint_idx - 1],
                        value: values[hint_idx - 1],
                    });
                    let following = None;
                    (preceding, following)
                } else {
                    // The sample should exist within the signal.
                    assert!(time_points.len() >= 2, "There should be at least 2 elements");
                    let preceding = Some(Sample {
                        time: time_points[hint_idx - 1],
                        value: values[hint_idx - 1],
                    });
                    let following = Some(Sample {
                        time: time_points[hint_idx],
                        value: values[hint_idx],
                    });
                    (preceding, following)
                };

                interp.at(time, &first, &second)
            }
        }
    }

    pub fn time_points(&self) -> Option<Vec<&Duration>> {
        match self {
            Signal::Empty => None,
            Signal::Constant { value: _ } => Vec::new().into(),
            Signal::Sampled { values: _, time_points } => time_points.iter().collect_vec().into(),
        }
    }

    /// Return a list consisting of all the points where the two signals should be
    /// sampled and synchronized for operations.
    pub fn sync_points<'a>(&'a self, other: &'a Self) -> Option<Vec<&'a Duration>> {
        use core::ops::Bound::*;

        if self.is_empty() || other.is_empty() {
            return None;
        }
        match (self, other) {
            (Signal::Empty, _) | (_, Signal::Empty) => None,
            (Signal::Constant { value: _ }, Signal::Constant { value: _ }) => Vec::new().into(),
            (Signal::Constant { value: _ }, Signal::Sampled { values: _, time_points })
            | (Signal::Sampled { values: _, time_points }, Signal::Constant { value: _ }) => {
                time_points.iter().collect_vec().into()
            }
            (
                Signal::Sampled {
                    values: _,
                    time_points: lhs,
                },
                Signal::Sampled {
                    values: _,
                    time_points: rhs,
                },
            ) => {
                let bounds = match intersect_bounds(&self.bounds()?, &other.bounds()?) {
                    (Included(start), Included(end)) => start..=end,
                    (..) => unreachable!(),
                };

                itertools::merge(lhs, rhs)
                    .filter(|time| bounds.contains(time))
                    .dedup()
                    .collect_vec()
                    .into()
            }
        }
    }

    /// Augment synchronization points with time points where signals intersect
    pub fn sync_with_intersection(&self, other: &Signal<T>) -> Option<Vec<Duration>>
    where
        T: PartialOrd + Copy + LinearInterpolatable + NumCast,
    {
        use core::cmp::Ordering::*;
        let sync_points: Vec<&Duration> = self.sync_points(other)?.into_iter().collect();
        // This will contain the new signal with an initial capacity of twice the input
        // signals sample points (as that is the upper limit of the number of new points
        // that will be added
        let mut return_points = Vec::<Duration>::with_capacity(sync_points.len() * 2);
        // this will contain the last sample point and ordering
        let mut last_sample = None;
        // We will now loop over the sync points, compare across signals and (if
        // an intersection happens) we will have to compute the intersection point
        for t in sync_points {
            let lhs = self.at(*t).expect("value must be present at given time");
            let rhs = other.at(*t).expect("values must be present at given time");
            let ord = lhs.partial_cmp(rhs).unwrap();

            // We will check for any intersections between the current sample and the
            // previous one before we push the current sample time
            if let Some((tm1, last)) = last_sample {
                // Check if the signals crossed, this will happen essentially if the last
                // and the current are opposites and were not Equal.
                if let (Less, Greater) | (Greater, Less) = (last, ord) {
                    // Find the point of intersection between the points.
                    let a = utils::Neighborhood {
                        first: self.at(tm1).copied().map(|value| Sample { time: tm1, value }),
                        second: self.at(*t).copied().map(|value| Sample { time: *t, value }),
                    };
                    let b = utils::Neighborhood {
                        first: other.at(tm1).copied().map(|value| Sample { time: tm1, value }),
                        second: other.at(*t).copied().map(|value| Sample { time: *t, value }),
                    };
                    let intersect = utils::find_intersection(&a, &b);
                    return_points.push(intersect.time);
                }
            }
            return_points.push(*t);
            last_sample = Some((*t, ord));
        }
        return_points.shrink_to_fit();
        Some(return_points)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
pub mod arbitrary {
    use proptest::prelude::*;
    use proptest::sample::SizeRange;

    use super::*;

    /// Generate an arbitrary list of samples and two indices within the list
    pub fn samples_and_indices<T>(
        size: impl Into<SizeRange>,
    ) -> impl Strategy<Value = (Vec<(Duration, T)>, usize, usize)>
    where
        T: Arbitrary + Copy,
    {
        samples(size).prop_flat_map(|vec| {
            let len = vec.len();
            if len == 0 {
                (Just(vec), 0..1, 0..1)
            } else {
                (Just(vec), 0..len, 0..len)
            }
        })
    }

    /// Generate arbitrary samples for a signal where the time stamps are strictly
    /// monotonically increasing
    pub fn samples<T>(size: impl Into<SizeRange>) -> impl Strategy<Value = Vec<(Duration, T)>>
    where
        T: Arbitrary + Copy,
    {
        prop::collection::vec(any::<T>(), size).prop_flat_map(|values| {
            let len = values.len();
            prop::collection::vec(any::<u64>(), len).prop_map(move |mut ts| {
                ts.sort_unstable();
                ts.dedup();
                ts.into_iter()
                    .map(Duration::from_secs)
                    .zip(values.clone().into_iter())
                    .collect::<Vec<_>>()
            })
        })
    }

    /// Generate arbitrary finite-length signals with samples of the given type
    pub fn sampled_signal<T>(size: impl Into<SizeRange>) -> impl Strategy<Value = Signal<T>>
    where
        T: Arbitrary + Copy,
    {
        samples(size).prop_map(Signal::<T>::from_iter)
    }

    /// Generate an arbitrary constant signal
    pub fn constant_signal<T>() -> impl Strategy<Value = Signal<T>>
    where
        T: Arbitrary + Copy,
    {
        any::<T>().prop_map(Signal::constant)
    }

    /// Generate an arbitrary signal
    pub fn signal<T>(size: impl Into<SizeRange>) -> impl Strategy<Value = Signal<T>>
    where
        T: Arbitrary + Copy,
    {
        prop_oneof![constant_signal::<T>(), sampled_signal::<T>(size),]
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Bound;

    use paste::paste;
    use proptest::prelude::*;

    use super::*;

    macro_rules! correctly_create_signals_impl {
        ($ty:ty) => {
            proptest! {
                |((samples, idx, _) in arbitrary::samples_and_indices::<$ty>(0..100))| {
                    // Creating a signal should be fine
                    let signal: Signal<_> = samples.clone().into_iter().collect();

                    if samples.len() > 0 {
                        // We wil get the start and end times.
                        let start_time = samples.first().unwrap().0;
                        let end_time = samples.last().unwrap().0;
                        // Get the value of the sample at a given index
                        let (at, val) = samples[idx];

                        assert_eq!(signal.start_time(), Some(Bound::Included(start_time)));
                        assert_eq!(signal.end_time(), Some(Bound::Included(end_time)));
                        assert_eq!(signal.at(at), Some(&val));
                        assert_eq!(signal.at(end_time + Duration::from_secs(1)), None);
                        assert_eq!(signal.at(start_time - Duration::from_secs(1)), None);
                    } else {
                        assert!(signal.is_empty());
                        assert_eq!(signal.at(Duration::from_secs(1)), None);
                    }
                }
            }

            proptest! {
                |((mut samples, a, b) in arbitrary::samples_and_indices::<$ty>(5..100))| {
                    prop_assume!(a != b);
                    // Swap two indices in the samples
                    samples.swap(a, b);
                    // Creating a signal should fail
                    let signal = Signal::try_from_iter(samples.clone());
                    assert!(signal.is_err(), "swapped {:?} and {:?}", samples[a], samples[b]);
                }
            }
        };
    }

    #[test]
    fn create_signals_from_samples() {
        correctly_create_signals_impl!(bool);
        correctly_create_signals_impl!(i8);
        correctly_create_signals_impl!(i16);
        correctly_create_signals_impl!(i32);
        correctly_create_signals_impl!(i64);
        correctly_create_signals_impl!(u8);
        correctly_create_signals_impl!(u16);
        correctly_create_signals_impl!(u32);
        correctly_create_signals_impl!(u64);
        correctly_create_signals_impl!(f32);
        correctly_create_signals_impl!(f64);
    }

    macro_rules! signals_fromiter_panic {
        ($ty:ty) => {
            paste! {
                proptest! {
                    #[test]
                    #[should_panic]
                    fn [<fail_create_ $ty _signal>] ((mut samples, a, b) in arbitrary::samples_and_indices::<$ty>(5..100))
                    {
                        prop_assume!(a != b);
                        // Swap two indices in the samples
                        samples.swap(a, b);
                        // Creating a signal should fail
                        let _: Signal<_> = samples.into_iter().collect();
                    }
                }
            }

        };
    }

    signals_fromiter_panic!(bool);
    signals_fromiter_panic!(i8);
    signals_fromiter_panic!(i16);
    signals_fromiter_panic!(i32);
    signals_fromiter_panic!(i64);
    signals_fromiter_panic!(u8);
    signals_fromiter_panic!(u16);
    signals_fromiter_panic!(u32);
    signals_fromiter_panic!(u64);
    signals_fromiter_panic!(f32);
    signals_fromiter_panic!(f64);

    macro_rules! signal_ops_impl {
        ($ty:ty, $op:tt sig) => {
            proptest! {
                |(sig in arbitrary::sampled_signal::<$ty>(1..100))| {
                    use InterpolationMethod::Linear;
                    let new_sig = $op (&sig);
                    for (t, v) in new_sig.iter() {
                        let prev = sig.interpolate_at(*t, Linear).unwrap();
                        assert_eq!($op prev, *v);
                    }
                }
            }
        };
        ($ty:ty, lhs $op:tt rhs) => {
            proptest! {
                |(sig1 in arbitrary::sampled_signal::<$ty>(1..100), sig2 in arbitrary::sampled_signal::<$ty>(1..100))| {
                    use InterpolationMethod::Linear;
                    let new_sig = &sig1 $op &sig2;
                    for (t, v) in new_sig.iter() {
                        let v1 = sig1.interpolate_at(*t, Linear).unwrap();
                        let v2 = sig2.interpolate_at(*t, Linear).unwrap();
                        assert_eq!(v1 $op v2, *v);
                    }
                }
            }

            proptest! {
                |(sig1 in arbitrary::sampled_signal::<$ty>(1..100), sig2 in arbitrary::constant_signal::<$ty>())| {
                    use InterpolationMethod::Linear;
                    let new_sig = &sig1 $op &sig2;
                    for (t, v) in new_sig.iter() {
                        let v1 = sig1.interpolate_at(*t, Linear).unwrap();
                        let v2 = sig2.interpolate_at(*t, Linear).unwrap();
                        assert_eq!(v1 $op v2, *v);
                    }
                }
            }

            proptest! {
                |(sig1 in arbitrary::constant_signal::<$ty>(), sig2 in arbitrary::constant_signal::<$ty>())| {
                    let new_sig = &sig1 $op &sig2;
                    match (sig1, sig2, new_sig) {
                        (Signal::Constant { value: v1 }, Signal::Constant { value: v2 }, Signal::Constant { value: v }) => assert_eq!(v1 $op v2, v),
                        (s1, s2, s3) => panic!("{:?}, {:?} = {:?}", s1, s2, s3),
                    }
                }
            }
        };
    }

    #[test]
    fn signal_ops() {
        signal_ops_impl!(bool, !sig);
        signal_ops_impl!(bool, lhs | rhs);
        signal_ops_impl!(bool, lhs & rhs);

        // signal_ops_impl!(u64, lhs + rhs);
        // signal_ops_impl!(u64, lhs * rhs);
        signal_ops_impl!(u64, lhs / rhs);

        signal_ops_impl!(i64, -sig);
        // signal_ops_impl!(i64, lhs + rhs);
        // signal_ops_impl!(i64, lhs * rhs);
        signal_ops_impl!(i64, lhs / rhs);

        signal_ops_impl!(f32, -sig);
        signal_ops_impl!(f32, lhs + rhs);
        signal_ops_impl!(f32, lhs * rhs);
        // signal_ops_impl!(f32, lhs / rhs);

        signal_ops_impl!(f64, -sig);
        signal_ops_impl!(f64, lhs + rhs);
        signal_ops_impl!(f64, lhs * rhs);
        // signal_ops_impl!(f64, lhs / rhs);
    }
}
