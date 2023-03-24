//! A bunch of utility code for argus
//!
//! - The implementation for Range intersection is based on the library
//! [`range_ext`](https://github.com/AnickaBurova/range-ext), but adapted for my use a
//! bit.

use core::ops::{Bound, RangeBounds};
use core::time::Duration;

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
    T: Copy + std::fmt::Debug + NumCast,
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

pub fn apply1<T, F>(signal: &Signal<T>, op: F) -> Signal<T>
where
    T: Copy,
    F: Fn(T) -> T,
    Signal<T>: std::iter::FromIterator<(Duration, T)>,
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

/// More precise intersection of two ranges from point of the first range
#[derive(Debug, PartialEq)]
pub enum Intersection {
    /// The self is below the other
    Below,
    /// The self is below but overlaping
    BelowOverlap,
    /// The self is within the other
    Within,
    /// The self is same as the other
    Same,
    /// The self is over the other, the other is within the self
    Over,
    /// The self is above but overlaping
    AboveOverlap,
    /// The self is above the other
    Above,
}

impl Intersection {
    /// Test if there is any intersection
    pub fn is_any(&self) -> bool {
        !matches!(self, Intersection::Below | Intersection::Above)
    }
    /// Test if the range is fully within the other
    pub fn is_within(&self) -> bool {
        matches!(self, Intersection::Within | Intersection::Same)
    }

    /// Test if the range is fully over the other
    pub fn is_over(&self) -> bool {
        matches!(self, Intersection::Over | Intersection::Same)
    }
}

pub trait Intersect<T, U>
where
    T: PartialOrd,
    U: RangeBounds<T>,
{
    /// Test two ranges for an intersection
    fn check_intersect(&self, other: &U) -> Intersection;
}

impl<T, U, R> Intersect<T, U> for R
where
    T: PartialOrd + PartialEq,
    U: RangeBounds<T>,
    R: RangeBounds<T>,
{
    fn check_intersect(&self, other: &U) -> Intersection {
        use core::cmp::Ordering::*;
        use core::ops::Bound::*;

        // We find where the start of self is with respect to that of other
        let (left_rel_pos, me_start) = match (self.start_bound(), other.start_bound()) {
            (Included(me), Excluded(them)) if me == them => (Less, Some(me)), // [a, _} left of (a, }
            //
            (Excluded(me), Included(them)) if me == them => (Greater, Some(me)), // (a, _} right of [a, }

            // If both are consistently open or close, or they are not equal then we
            // just compare them
            (Included(me), Excluded(them))
            | (Excluded(me), Included(them))
            | (Included(me), Included(them))
            | (Excluded(me), Excluded(them)) => (me.partial_cmp(them).unwrap(), Some(me)),

            // start of self > start of other
            (Included(me), Unbounded) | (Excluded(me), Unbounded) => (Greater, Some(me)),

            (Unbounded, Unbounded) => (Equal, None), // unbounded start

            (Unbounded, _) => (Less, None), // start of self < start of other
        };

        // We find where the end of self is with respect to that of other
        let (right_rel_pos, me_end) = match (self.end_bound(), other.end_bound()) {
            (Included(me), Excluded(them)) if me == them => (Greater, Some(me)), // {_, a] right of {_, a)

            (Excluded(me), Included(them)) if me == them => (Less, Some(me)), // {_, a) right of {_, a]

            // If both are consistently open or close, or they are not equal then we just compare them
            (Included(me), Excluded(them))
            | (Excluded(me), Included(them))
            | (Included(me), Included(them))
            | (Excluded(me), Excluded(them)) => (me.partial_cmp(them).unwrap(), Some(me)),

            (Included(me), Unbounded) | (Excluded(me), Unbounded) => (Less, Some(me)), // end of self < end of other

            (Unbounded, Unbounded) => (Equal, None), // unbounded end

            (Unbounded, _) => (Greater, None), // end of self > end of other
        };

        // We have gotten the relative position of the ends. But we need to check if one
        // of the ends are contained within the bounds of the other.

        match (left_rel_pos, right_rel_pos) {
            (Less, Less) => {
                // Check if the end of self is contained within other's domain
                // NOTE: Since right is less than, me_end must not be None
                assert!(me_end.is_some());
                if other.contains(me_end.unwrap()) {
                    // self is below but overlaps
                    Intersection::BelowOverlap
                } else {
                    // self is strictly below
                    Intersection::Below
                }
            }
            (Greater, Greater) => {
                // Check if the start of self is contained within other's domain
                // NOTE: Since left is greater than, me_start must not be None
                assert!(me_start.is_some());
                if other.contains(me_start.unwrap()) {
                    // self is to the right of but overlaps other
                    Intersection::AboveOverlap
                } else {
                    // self is strictly above
                    Intersection::Above
                }
            }
            (Less, Greater) | (Equal, Greater) | (Less, Equal) => Intersection::Over, // self contains other
            (Equal, Less) | (Greater, Equal) | (Greater, Less) => Intersection::Within, // self within other
            (Equal, Equal) => Intersection::Same,                                     // The ranges are equal
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn range_range_intersection(r1 in any::<Range<i32>>(), r2 in any::<Range<i32>>()) {
            use Intersection::*;
            let intersect_p = r1.check_intersect(&r2);
            match intersect_p {
                Below => assert!(r1.end < r2.start, "Expected strict below"),
                BelowOverlap => {
                    assert!(r1.start < r2.start, "Expected below with overlap");
                    assert!(r2.contains(&r1.end), "Expected below with overlap");
                },
                Within => {
                    assert!(r2.contains(&r1.end), "Expected to be contained");
                    assert!(r2.contains(&r1.start), "Expected to be contained");
                }
                Same => {
                    assert!(r1.start == r2.start, "Expected to be same");
                    assert!(r1.end == r2.end, "Expected to be same");
                }
                Over => {
                    assert!(r1.contains(&r2.start), "Expected to cover");
                    assert!(r1.contains(&r2.end), "Expected to cover");
                }
                AboveOverlap => {
                    assert!(r2.contains(&r1.start), "Expected above with overlap");
                    assert!(r1.end > r2.end, "Expected above with overlap");
                }
                Above => assert!(r1.start > r2.end, "Expected strict above"),
            }
        }
    }
}