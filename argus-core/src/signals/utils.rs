//! A bunch of utility code for argus
//!
//! - The implementation for Range intersection is based on the library
//! [`range_ext`](https://github.com/AnickaBurova/range-ext), but adapted for my use a
//! bit.

use core::ops::{Bound, RangeBounds};
use core::time::Duration;
use std::cmp::Ordering;

use num_traits::NumCast;

use super::traits::{LinearInterpolatable, SignalSyncPoints};
use super::{BaseSignal, ConstantSignal, InterpolationMethod, Sample, Signal};

/// The neighborhood around a signal such that the time `at` is between the `first` and
/// `second` samples.
///
/// The values of `first` and `second` are `None` if and only if `at` lies outside the
/// domain over which the signal is defined.
///
/// This can be used to interpolate the value at the given `at` time using strategies
/// like constant previous, constant following, and linear interpolation.
#[derive(Copy, Clone, Debug)]
pub struct Neighborhood<T: ?Sized + Copy> {
    pub first: Option<Sample<T>>,
    pub second: Option<Sample<T>>,
}

/// Given two signals with two sample points each, find the intersection of the two
/// lines.
pub fn find_intersection<T>(a: &Neighborhood<T>, b: &Neighborhood<T>) -> Sample<T>
where
    T: Copy + NumCast,
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
    let y: T = cast(y_top / denom).unwrap();
    Sample { time: t, value: y }
}

/// Augment synchronization points with time points where signals intersect
pub fn sync_with_intersection<'a, T, Sig1, Sig2>(sig1: &'a Sig1, sig2: &'a Sig2) -> Option<Vec<Duration>>
where
    T: PartialOrd + Copy + NumCast + LinearInterpolatable,
    Sig1: BaseSignal<Value = T> + SignalSyncPoints<Sig2>,
    Sig2: BaseSignal<Value = T> + SignalSyncPoints<Sig1>,
{
    use Ordering::*;
    let sync_points: Vec<&Duration> = sig1.synchronization_points(sig2)?.into_iter().collect();
    // This will contain the new signal with an initial capacity of twice the input
    // signals sample points (as that is the upper limit of the number of new points
    // that will be added
    let mut return_points = Vec::<Duration>::with_capacity(sync_points.len() * 2);
    // this will contain the last sample point and ordering
    let mut last_sample = None;
    // We will now loop over the sync points, compare across signals and (if
    // an intersection happens) we will have to compute the intersection point
    for t in sync_points {
        let lhs = sig1.at(*t).expect("value must be present at given time");
        let rhs = sig2.at(*t).expect("values must be present at given time");
        let ord = lhs.partial_cmp(rhs).unwrap();

        // We will check for any intersections between the current sample and the
        // previous one before we push the current sample time
        if let Some((tm1, last)) = last_sample {
            // Check if the signals crossed, this will happen essentially if the last
            // and the current are opposites and were not Equal.
            if let (Less, Greater) | (Greater, Less) = (last, ord) {
                // Find the point of intersection between the points.
                let a = Neighborhood {
                    first: sig1.at(tm1).copied().map(|value| Sample { time: tm1, value }),
                    second: sig1.at(*t).copied().map(|value| Sample { time: *t, value }),
                };
                let b = Neighborhood {
                    first: sig2.at(tm1).copied().map(|value| Sample { time: tm1, value }),
                    second: sig2.at(*t).copied().map(|value| Sample { time: *t, value }),
                };
                let intersect = find_intersection(&a, &b);
                return_points.push(intersect.time);
            }
        }
        return_points.push(*t);
        last_sample = Some((*t, ord));
    }
    return_points.shrink_to_fit();
    Some(return_points)
}

pub fn apply1<T, U, F>(signal: &Signal<T>, op: F) -> Signal<U>
where
    T: Copy,
    F: Fn(T) -> U,
    Signal<U>: std::iter::FromIterator<(Duration, U)>,
{
    signal.iter().map(|(t, v)| (*t, op(*v))).collect()
}

pub fn apply2<'a, T, U, F>(lhs: &'a Signal<T>, rhs: &'a Signal<T>, op: F) -> Signal<U>
where
    T: Copy + LinearInterpolatable,
    U: Copy,
    F: Fn(T, T) -> U,
{
    // If either of the signals are empty, we return an empty signal.
    if lhs.is_empty() || rhs.is_empty() {
        // Intersection with empty signal should yield an empty signal
        return Signal::<U>::new();
    }
    // We determine the range of the signal (as the output signal can only be
    // defined in the domain where both signals are defined).
    let time_points = lhs.synchronization_points(rhs).unwrap();
    // Now, at each of the merged time points, we sample each signal and operate on
    // them
    time_points
        .into_iter()
        .map(|t| {
            let v1 = lhs.interpolate_at(*t, InterpolationMethod::Linear).unwrap();
            let v2 = rhs.interpolate_at(*t, InterpolationMethod::Linear).unwrap();
            (*t, op(v1, v2))
        })
        .collect()
}

pub fn apply2_const<'a, T, U, F>(lhs: &'a Signal<T>, rhs: &'a ConstantSignal<T>, op: F) -> Signal<U>
where
    T: Copy + LinearInterpolatable,
    U: Copy,
    F: Fn(T, T) -> U,
{
    // If either of the signals are empty, we return an empty signal.
    if lhs.is_empty() {
        // Intersection with empty signal should yield an empty signal
        return Signal::<U>::new();
    }
    lhs.time_points
        .iter()
        .map(|&t| {
            let v1 = lhs.interpolate_at(t, InterpolationMethod::Linear).unwrap();
            let v2 = rhs.interpolate_at(t, InterpolationMethod::Linear).unwrap();
            (t, op(v1, v2))
        })
        .collect()
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
