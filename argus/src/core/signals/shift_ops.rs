use core::iter::zip;
use core::time::Duration;

use itertools::Itertools;

use super::{InterpolationMethod, Signal};

impl<T> Signal<T>
where
    T: Copy,
{
    /// Shift all samples in the signal by `delta` amount to the left.
    ///
    /// This essentially filters out all samples with time points greater than `delta`,
    /// and subtracts `delta` from the rest of the time points.
    pub fn shift_left<I: InterpolationMethod<T>>(&self, delta: Duration) -> Self {
        match self {
            Signal::Sampled { values, time_points } => {
                // We want to skip any time points < delta, and subtract the rest.
                // Moreover, if the signal doesn't start at 0 after the shift, we may
                // want to interpolate from the previous value.

                // Find the first index that satisfies `t >= delta` while also checking
                // if we need to interpolate
                let Some((idx, first_t)) = time_points.iter().find_position(|&t| t >= &delta) else {
                    // Return an empty signal (we exhauseted all samples).
                    return Signal::Empty;
                };

                let mut new_samples: Vec<(Duration, T)> = Vec::with_capacity(time_points.len() - idx);
                // Let's see if we need to find a new sample
                if idx > 0 && first_t != &delta {
                    // The shifted signal will not start at 0, and we have a previous
                    // index to interpolate from.
                    let v = self.interpolate_at::<I>(delta).unwrap();
                    new_samples.push((Duration::ZERO, v));
                }
                // Shift the rest of the samples
                new_samples.extend(zip(&time_points[idx..], &values[idx..]).map(|(&t, &v)| (t - delta, v)));
                new_samples.into_iter().collect()
            }
            // Empty and constant signals can't really be changed
            sig => sig.clone(),
        }
    }

    /// Shift all samples in the signal by `delta` amount to the right.
    ///
    /// This essentially adds `delta` to all time points.
    pub fn shift_right(&self, delta: Duration) -> Self {
        match self {
            Signal::Sampled { values, time_points } => {
                zip(time_points, values).map(|(&t, &v)| (t + delta, v)).collect()
            }
            // Empty and constant signals can't really be changed
            sig => sig.clone(),
        }
    }
}
