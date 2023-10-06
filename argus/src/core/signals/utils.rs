//! A bunch of utility code for argus
//!
//! - The implementation for Range intersection is based on the library
//! [`range_ext`](https://github.com/AnickaBurova/range-ext), but adapted for my use a
//! bit.

use core::ops::{Bound, RangeBounds};
use core::time::Duration;

use super::{InterpolationMethod, Sample, Signal};

/// The neighborhood around a signal such that the time `at` is between the `first` and
/// `second` samples.
///
/// The values of `first` and `second` are `None` if and only if `at` lies outside the
/// domain over which the signal is defined.
///
/// This can be used to interpolate the value at the given `at` time using strategies
/// like constant previous, constant following, and linear interpolation.
#[derive(Copy, Clone, Debug)]
pub struct Neighborhood<T> {
    pub first: Option<Sample<T>>,
    pub second: Option<Sample<T>>,
}

impl<T> Signal<T> {
    pub(crate) fn unary_op<U, F>(&self, op: F) -> Signal<U>
    where
        F: Fn(&T) -> U,
        Signal<U>: std::iter::FromIterator<(Duration, U)>,
    {
        use Signal::*;
        match self {
            Empty => Signal::Empty,
            Constant { value } => Signal::constant(op(value)),
            signal => signal.into_iter().map(|(&t, v)| (t, op(v))).collect(),
        }
    }

    pub(crate) fn binary_op<U, F, Interp>(&self, other: &Signal<T>, op: F) -> Signal<U>
    where
        T: Clone,
        F: Fn(&T, &T) -> U,
        Interp: InterpolationMethod<T>,
    {
        use Signal::*;
        match (self, other) {
            // If either of the signals are empty, we return an empty signal.
            (Empty, _) | (_, Empty) => Signal::Empty,
            (Constant { value: v1 }, Constant { value: v2 }) => Signal::constant(op(v1, v2)),
            (lhs, rhs) => {
                // We determine the range of the signal (as the output signal can only be
                // defined in the domain where both signals are defined).
                let time_points = lhs.sync_points(rhs).unwrap();
                // Now, at each of the merged time points, we sample each signal and operate on
                // them
                time_points
                    .into_iter()
                    .map(|t| {
                        let v1 = lhs.interpolate_at::<Interp>(*t).unwrap();
                        let v2 = rhs.interpolate_at::<Interp>(*t).unwrap();
                        (*t, op(&v1, &v2))
                    })
                    .collect()
            }
        }
    }

    pub(crate) fn binary_op_with_intersection<U, F, Interp>(&self, other: &Signal<T>, op: F) -> Signal<U>
    where
        T: Clone + PartialOrd,
        F: Fn(&T, &T) -> U,
        Interp: InterpolationMethod<T>,
    {
        use Signal::*;
        match (self, other) {
            // If either of the signals are empty, we return an empty signal.
            (Empty, _) | (_, Empty) => Signal::Empty,
            (Constant { value: v1 }, Constant { value: v2 }) => Signal::constant(op(v1, v2)),
            (lhs, rhs) => {
                // We determine the range of the signal (as the output signal can only be
                // defined in the domain where both signals are defined).
                let time_points = lhs.sync_with_intersection::<Interp>(rhs).unwrap();
                // Now, at each of the merged time points, we sample each signal and operate on
                // them
                time_points
                    .into_iter()
                    .map(|t| {
                        let v1 = lhs.interpolate_at::<Interp>(t).unwrap();
                        let v2 = rhs.interpolate_at::<Interp>(t).unwrap();
                        (t, op(&v1, &v2))
                    })
                    .collect()
            }
        }
    }
}

fn partial_min<T>(a: T, b: T) -> Option<T>
where
    T: PartialOrd,
{
    a.partial_cmp(&b).map(|ord| if ord.is_lt() { a } else { b })
}

fn partial_max<T>(a: T, b: T) -> Option<T>
where
    T: PartialOrd,
{
    a.partial_cmp(&b).map(|ord| if ord.is_gt() { a } else { b })
}

/// Compute the intersection of two ranges
pub fn intersect_bounds<T>(lhs: &impl RangeBounds<T>, rhs: &impl RangeBounds<T>) -> (Bound<T>, Bound<T>)
where
    T: PartialOrd + Copy,
{
    use core::ops::Bound::*;

    let start = match (lhs.start_bound(), rhs.start_bound()) {
        (Included(&l), Included(&r)) => Included(partial_max(l, r).unwrap()),
        (Excluded(&l), Excluded(&r)) => Excluded(partial_max(l, r).unwrap()),

        (Included(l), Excluded(r)) | (Excluded(r), Included(l)) => {
            if l > r {
                Included(*l)
            } else {
                Excluded(*r)
            }
        }

        (Unbounded, Included(&l)) | (Included(&l), Unbounded) => Included(l),
        (Unbounded, Excluded(&l)) | (Excluded(&l), Unbounded) => Excluded(l),
        (Unbounded, Unbounded) => Unbounded,
    };

    let end = match (lhs.end_bound(), rhs.end_bound()) {
        (Included(&l), Included(&r)) => Included(partial_min(l, r).unwrap()),
        (Excluded(&l), Excluded(&r)) => Excluded(partial_min(l, r).unwrap()),

        (Included(l), Excluded(r)) | (Excluded(r), Included(l)) => {
            if l < r {
                Included(*l)
            } else {
                Excluded(*r)
            }
        }

        (Unbounded, Included(&l)) | (Included(&l), Unbounded) => Included(l),
        (Unbounded, Excluded(&l)) | (Excluded(&l), Unbounded) => Excluded(l),
        (Unbounded, Unbounded) => Unbounded,
    };

    (start, end)
}
