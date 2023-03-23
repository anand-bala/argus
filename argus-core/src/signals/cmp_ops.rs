use std::{cmp::Ordering, time::Duration};

use num_traits::NumCast;

use crate::signals::{
    utils::{find_intersection, Neighborhood},
    InterpolationMethod, Sample,
};

use super::{
    traits::{BaseSignal, LinearInterpolatable, SignalPartialOrd, SignalSyncPoints},
    ConstantSignal, Signal,
};

fn sync_with_intersection<'a, T, Sig1, Sig2, F>(
    sig1: &'a Sig1,
    sig2: &'a Sig2,
    sync_points: &[&'a Duration],
    op: F,
) -> Signal<bool>
where
    F: Fn(Ordering) -> bool,
    T: PartialOrd + Copy + std::fmt::Debug + NumCast + LinearInterpolatable,
    Sig1: BaseSignal<Value = T>,
    Sig2: BaseSignal<Value = T>,
{
    // This has to be manually implemented and cannot use the apply2 functions.
    // This is because if we have two signals that cross each other, then there is
    // an intermediate point where the two signals are equal. This point must be
    // added to the signal appropriately.
    use Ordering::*;
    // This will contain the new signal with an initial capacity of twice the input
    // signals sample points (as that is the upper limit of the number of new points
    // that will be added
    let mut return_signal = Signal::<bool>::new_with_capacity(sync_points.len() * 2);
    // this will contain the last sample point and ordering
    let mut last_sample = None;
    // We will now loop over the sync points, compare across signals and (if
    // an intersection happens) we will have to compute the intersection point
    for &t in sync_points {
        let lhs = sig1.at(*t).expect("value must be present at given time");
        let rhs = sig2.at(*t).expect("values must be present at given time");
        let ord = lhs.partial_cmp(rhs).unwrap();

        if let Some((tm1, last)) = last_sample {
            // Check if the signals crossed, this will happen essentiall if the last
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
                {
                    let lhs = sig1
                        .interpolate_at(intersect.time, InterpolationMethod::Linear)
                        .unwrap();
                    let rhs = sig2
                        .interpolate_at(intersect.time, InterpolationMethod::Linear)
                        .unwrap();
                    assert_eq!(lhs, rhs);
                }
                return_signal
                    .push(intersect.time, op(Equal))
                    .expect("Signal should already be monotonic");
            }
        }
        last_sample = Some((*t, ord));
    }
    return_signal.time_points.shrink_to_fit();
    return_signal.values.shrink_to_fit();
    return_signal
}

impl<T> SignalPartialOrd<Self> for Signal<T>
where
    T: PartialOrd + Copy + std::fmt::Debug + NumCast + LinearInterpolatable,
{
    type Output = Signal<bool>;

    fn signal_cmp<F>(&self, other: &Self, op: F) -> Option<Self::Output>
    where
        F: Fn(Ordering) -> bool,
    {
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = match self.synchronization_points(other) {
            Some(points) => points,
            None => return None,
        };

        Some(sync_with_intersection(self, other, &sync_points, op))
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
        // the union of the sample points in self and other
        let sync_points = match self.synchronization_points(other) {
            Some(points) => points,
            None => return None,
        };

        Some(sync_with_intersection(self, other, &sync_points, op))
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
