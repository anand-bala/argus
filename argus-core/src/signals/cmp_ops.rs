use std::cmp::Ordering;

use super::interpolation::Linear;
use super::traits::{SignalMinMax, SignalPartialOrd};
use super::{FindIntersectionMethod, InterpolationMethod, Signal};

impl<T> SignalPartialOrd<Self> for Signal<T>
where
    T: PartialOrd + Copy,
    Linear: InterpolationMethod<T> + FindIntersectionMethod<T>,
{
    fn signal_cmp<F>(&self, other: &Self, op: F) -> Option<Signal<bool>>
    where
        F: Fn(Ordering) -> bool,
    {
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = self.sync_with_intersection::<Linear>(other)?;
        let sig: Option<Signal<bool>> = sync_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<Linear>(t).unwrap();
                let rhs = other.interpolate_at::<Linear>(t).unwrap();
                let cmp = lhs.partial_cmp(&rhs);
                cmp.map(|v| (t, op(v)))
            })
            .collect();
        sig
    }
}

impl<T> SignalMinMax<Self> for Signal<T>
where
    T: PartialOrd + Copy,
    Linear: InterpolationMethod<T> + FindIntersectionMethod<T>,
{
    type Output = Signal<T>;

    fn min(&self, other: &Self) -> Self::Output {
        let time_points = self.sync_with_intersection::<Linear>(other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<Linear>(t).unwrap();
                let rhs = other.interpolate_at::<Linear>(t).unwrap();
                if lhs < rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }

    fn max(&self, other: &Self) -> Self::Output {
        let time_points = self.sync_with_intersection::<Linear>(other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<Linear>(t).unwrap();
                let rhs = other.interpolate_at::<Linear>(t).unwrap();
                if lhs > rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }
}
