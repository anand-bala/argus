use std::cmp::Ordering;

use num_traits::NumCast;

use super::traits::{BaseSignal, LinearInterpolatable, SignalMinMax, SignalPartialOrd};
use super::utils::sync_with_intersection;
use super::{ConstantSignal, InterpolationMethod, Signal};

impl<T> SignalPartialOrd<Self> for Signal<T>
where
    T: PartialOrd + Copy + std::fmt::Debug + NumCast + LinearInterpolatable,
{
    type Output = Signal<bool>;

    fn signal_cmp<F>(&self, other: &Self, op: F) -> Option<Self::Output>
    where
        F: Fn(Ordering) -> bool,
    {
        use super::InterpolationMethod::Linear;
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = sync_with_intersection(self, other)?;
        let sig: Signal<bool> = sync_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at(t, Linear).unwrap();
                let rhs = other.interpolate_at(t, Linear).unwrap();
                (t, op(lhs.partial_cmp(&rhs).unwrap()))
            })
            .collect();
        Some(sig)
    }
}

impl<T> SignalPartialOrd<ConstantSignal<T>> for Signal<T>
where
    T: PartialOrd + Copy + std::fmt::Debug + NumCast + LinearInterpolatable,
{
    type Output = Signal<bool>;

    fn signal_cmp<F>(&self, other: &ConstantSignal<T>, op: F) -> Option<Self::Output>
    where
        F: Fn(Ordering) -> bool,
    {
        use super::InterpolationMethod::Linear;
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = sync_with_intersection(self, other)?;
        let sig: Signal<bool> = sync_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at(t, Linear).unwrap();
                let rhs = other.interpolate_at(t, Linear).unwrap();
                (t, op(lhs.partial_cmp(&rhs).unwrap()))
            })
            .collect();
        Some(sig)
    }
}

impl<T> SignalPartialOrd<ConstantSignal<T>> for ConstantSignal<T>
where
    T: PartialOrd + Copy + std::fmt::Debug + NumCast,
{
    type Output = ConstantSignal<bool>;

    fn signal_cmp<F>(&self, other: &ConstantSignal<T>, op: F) -> Option<Self::Output>
    where
        F: Fn(Ordering) -> bool,
    {
        self.value.partial_cmp(&other.value).map(op).map(ConstantSignal::new)
    }
}

impl<T> SignalPartialOrd<Signal<T>> for ConstantSignal<T>
where
    T: PartialOrd + Copy + std::fmt::Debug + NumCast + LinearInterpolatable,
{
    type Output = Signal<bool>;

    fn signal_cmp<F>(&self, other: &Signal<T>, op: F) -> Option<Self::Output>
    where
        F: Fn(Ordering) -> bool,
    {
        other.signal_cmp(self, op)
    }
}

impl<T> SignalMinMax for ConstantSignal<T>
where
    T: PartialOrd + Copy,
{
    type Output = ConstantSignal<T>;

    fn min(&self, rhs: &Self) -> Self::Output {
        let value = if self.value < rhs.value { self.value } else { rhs.value };
        ConstantSignal::new(value)
    }

    fn max(&self, rhs: &Self) -> Self::Output {
        let value = if self.value > rhs.value { self.value } else { rhs.value };
        ConstantSignal::new(value)
    }
}

impl<T> SignalMinMax for Signal<T>
where
    T: PartialOrd + Copy + num_traits::NumCast + LinearInterpolatable,
{
    type Output = Signal<T>;

    fn min(&self, other: &Self) -> Self::Output {
        let time_points = sync_with_intersection(self, other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at(t, InterpolationMethod::Linear).unwrap();
                let rhs = other.interpolate_at(t, InterpolationMethod::Linear).unwrap();
                if lhs < rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }

    fn max(&self, other: &Self) -> Self::Output {
        let time_points = sync_with_intersection(self, other).unwrap();
        time_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at(t, InterpolationMethod::Linear).unwrap();
                let rhs = other.interpolate_at(t, InterpolationMethod::Linear).unwrap();
                if lhs > rhs {
                    (t, lhs)
                } else {
                    (t, rhs)
                }
            })
            .collect()
    }
}
