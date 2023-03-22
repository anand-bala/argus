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
pub mod iter;
pub mod traits;

use std::ops::{RangeFull, RangeInclusive};
use std::time::Duration;

use self::traits::{BaseSignal, LinearInterpolatable};
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

/// A signal is a sequence of time points ([`Duration`](core::time::Duration)) and
/// corresponding value samples.
#[derive(Default, Debug, Clone)]
pub struct Signal<T> {
    pub(crate) values: Vec<T>,
    pub(crate) time_points: Vec<Duration>,
}

impl<T> Signal<T> {
    /// Create a new empty signal
    pub fn new() -> Self {
        Self {
            values: Default::default(),
            time_points: Default::default(),
        }
    }

    /// Create a new empty signal with the specified capacity
    pub fn new_with_capacity(size: usize) -> Self {
        Self {
            values: Vec::with_capacity(size),
            time_points: Vec::with_capacity(size),
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
}

impl<T> BaseSignal for Signal<T> {
    type Value = T;
    type Bounds = RangeInclusive<Duration>;

    fn at(&self, time: Duration) -> Option<&Self::Value> {
        assert_eq!(
            self.time_points.len(),
            self.values.len(),
            "invariant: number of time points must equal number of samples"
        );
        // if there are no sample points, then there is no sample point (nor neighboring
        // sample points) to return
        if self.time_points.is_empty() {
            return None;
        }

        // We will use binary search to find the appropriate index
        match self.time_points.binary_search(&time) {
            Ok(idx) => self.values.get(idx),
            Err(_) => None,
        }
    }

    fn interpolate_at(&self, time: Duration, interp: InterpolationMethod) -> Option<Self::Value>
    where
        Self::Value: Copy + LinearInterpolatable,
    {
        assert_eq!(
            self.time_points.len(),
            self.values.len(),
            "invariant: number of time points must equal number of samples"
        );
        // if there are no sample points, then there is no sample point (nor neighboring
        // sample points) to return
        if self.time_points.is_empty() {
            return None;
        }

        // We will use binary search to find the appropriate index
        let hint_idx = match self.time_points.binary_search(&time) {
            Ok(idx) => return self.values.get(idx).copied(),
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
                time: self.time_points[hint_idx],
                value: self.values[hint_idx],
            });
            (preceding, following)
        } else if hint_idx == self.time_points.len() {
            // Sample appears past the end of the signal
            // So, let's return just the preceding sample, which is the last sample
            // (since we know the signal is non-empty)
            let preceding = Some(Sample {
                time: self.time_points[hint_idx - 1],
                value: self.values[hint_idx - 1],
            });
            let following = None;
            (preceding, following)
        } else {
            // The sample should exist within the signal.
            assert!(self.time_points.len() >= 2, "There should be at least 2 elements");
            let preceding = Some(Sample {
                time: self.time_points[hint_idx - 1],
                value: self.values[hint_idx - 1],
            });
            let following = Some(Sample {
                time: self.time_points[hint_idx],
                value: self.values[hint_idx],
            });
            (preceding, following)
        };

        interp.at(time, &first, &second)
    }

    fn bounds(&self) -> Self::Bounds {
        let first = self.time_points.first();
        let last = self.time_points.last();
        match (first, last) {
            (None, None) => Duration::from_secs(1)..=Duration::from_secs(0),
            (Some(first), Some(last)) => *first..=*last,
            (..) => unreachable!("there is either 0 time points or some time points"),
        }
    }

    fn push(&mut self, time: Duration, value: Self::Value) -> ArgusResult<bool> {
        assert_eq!(self.time_points.len(), self.values.len());

        let last_time = self.time_points.last();
        match last_time {
            Some(last_t) if last_t >= &time => Err(Error::NonMonotonicSignal {
                end_time: *last_t,
                current_sample: time,
            }),
            _ => {
                self.time_points.push(time);
                self.values.push(value);
                Ok(true)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantSignal<T> {
    pub value: T,
}

impl<T> ConstantSignal<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> BaseSignal for ConstantSignal<T> {
    type Value = T;

    type Bounds = RangeFull;

    fn at(&self, _time: Duration) -> Option<&Self::Value> {
        Some(&self.value)
    }

    fn bounds(&self) -> Self::Bounds {
        ..
    }

    fn interpolate_at(&self, _time: Duration, _interp: InterpolationMethod) -> Option<Self::Value>
    where
        Self::Value: Copy + LinearInterpolatable,
    {
        Some(self.value)
    }

    fn push(&mut self, _time: Duration, _value: Self::Value) -> ArgusResult<bool> {
        Ok(false)
    }
}

#[cfg(test)]
pub mod arbitrary {

    use itertools::Itertools;
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
                    .collect_vec()
            })
        })
    }

    /// Generate arbitrary  signals of the given type
    pub fn signal<T>(size: impl Into<SizeRange>) -> impl Strategy<Value = Signal<T>>
    where
        T: Arbitrary + Copy,
    {
        samples(size).prop_map(Signal::<T>::from_iter)
    }

    /// Generate an arbitrary constant signal
    pub fn constant_signal<T>() -> impl Strategy<Value = ConstantSignal<T>>
    where
        T: Arbitrary,
    {
        any::<T>().prop_map(ConstantSignal::new)
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

                        assert_eq!(signal.start_time(), Bound::Included(start_time));
                        assert_eq!(signal.end_time(), Bound::Included(end_time));
                        assert_eq!(signal.at(at), Some(&val));
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
}
