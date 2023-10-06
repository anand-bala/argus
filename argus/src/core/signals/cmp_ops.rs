use std::cmp::Ordering;

use super::traits::SignalPartialOrd;
use super::{InterpolationMethod, Signal};

impl<T> SignalPartialOrd<T> for Signal<T>
where
    T: PartialOrd + Clone,
{
    fn signal_cmp<F, I>(&self, other: &Self, op: F) -> Option<Signal<bool>>
    where
        F: Fn(Ordering) -> bool,
        I: InterpolationMethod<T>,
    {
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = self.sync_with_intersection::<I>(other)?;
        let sig: Option<Signal<bool>> = sync_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<I>(t).unwrap();
                let rhs = other.interpolate_at::<I>(t).unwrap();
                let cmp = lhs.partial_cmp(&rhs);
                cmp.map(|v| (t, op(v)))
            })
            .collect();
        sig
    }
}

impl<T> Signal<T>
where
    T: PartialOrd + Clone,
{
    /// Compute the time-wise min of two signals
    pub fn min<I>(&self, other: &Self) -> Self
    where
        I: InterpolationMethod<T>,
    {
        let time_points = self.sync_with_intersection::<I>(other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<I>(t).unwrap();
                let rhs = other.interpolate_at::<I>(t).unwrap();
                if lhs < rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }

    /// Compute the time-wise max of two signals
    pub fn max<I>(&self, other: &Self) -> Self
    where
        I: InterpolationMethod<T>,
    {
        let time_points = self.sync_with_intersection::<I>(other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at::<I>(t).unwrap();
                let rhs = other.interpolate_at::<I>(t).unwrap();
                if lhs > rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }
}
