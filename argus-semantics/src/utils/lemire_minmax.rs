//! A Rusty implementation of Alex Donze's adaptation[^1] of Daniel Lemire's streaming
//! min-max algorithm[^2] for piecewise linear signals.
//!
//! [^1]: Alexandre Donzé, Thomas Ferrère, and Oded Maler. 2013. Efficient Robust
//! Monitoring for STL. In Computer Aided Verification (Lecture Notes in Computer
//! Science), Springer, Berlin, Heidelberg, 264–279.
//!
//! [^2]: Daniel Lemire. 2007. Streaming Maximum-Minimum Filter Using No More than Three
//! Comparisons per Element. arXiv:cs/0610046.

use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MonoWedge<'a, T> {
    window: VecDeque<(&'a Duration, &'a T)>,
    duration: Duration,
    cmp: fn(&T, &T) -> bool,
}

impl<'a, T> MonoWedge<'a, T> {
    pub fn new(duration: Duration, cmp: fn(&T, &T) -> bool) -> Self {
        Self {
            window: Default::default(),
            cmp,
            duration,
        }
    }
}

impl<'a, T> MonoWedge<'a, T> {
    pub fn update(&mut self, sample: (&'a Duration, &'a T)) {
        assert!(
            self.window.back().map_or(true, |v| v.0 < sample.0),
            "MonoWedge window samples don't have monotonic time"
        );
        // Find the index to partition the inner queue based on the comparison function.
        let cmp_idx = self.window.partition_point(|a| (self.cmp)(a.1, sample.1));
        assert!(cmp_idx <= self.window.len());

        // And delete all items in the second partition.
        let _ = self.window.split_off(cmp_idx);

        // Clear all older values
        while let Some(item) = self.window.front() {
            if *sample.0 > self.duration + *item.0 {
                let _ = self.pop_front();
            } else {
                break;
            }
        }

        // Add the new value
        self.window.push_back(sample);
    }

    pub fn front(&self) -> Option<(&'a Duration, &'a T)> {
        self.window.front().copied()
    }

    pub fn pop_front(&mut self) -> Option<(&'a Duration, &'a T)> {
        self.window.pop_front()
    }
}

impl<'a, T> MonoWedge<'a, T>
where
    T: PartialOrd,
{
    pub fn min_wedge(duration: Duration) -> Self {
        Self::new(duration, T::lt)
    }

    pub fn max_wedge(duration: Duration) -> Self {
        Self::new(duration, T::gt)
    }
}

#[cfg(test)]
mod tests {

    use proptest::prelude::*;

    use super::*;

    prop_compose! {
        fn vec_and_window()(vec in prop::collection::vec(any::<u64>(), 3..100))
                        (window_size in 2..vec.len(), vec in Just(vec))
                        -> (Vec<u64>, usize) {
           (vec, window_size)
       }
    }

    proptest! {
        #[test]
        fn test_rolling_minmax((vec, width) in vec_and_window()) {
            // NOTE: When we convert the width from usize to Duration, the window becomes inclusive.
            let expected_mins: Vec<u64> = vec.as_slice().windows(width + 1).map(|w| w.iter().min().unwrap_or_else(|| panic!("slice should have min: {:?}", w))).copied().collect();
            assert_eq!(expected_mins.len(), vec.len() - width);
            let expected_maxs: Vec<u64> = vec.as_slice().windows(width + 1).map(|w| w.iter().max().unwrap_or_else(|| panic!("slice should have max: {:?}", w))).copied().collect();
            assert_eq!(expected_maxs.len(), vec.len() - width);

            let time_points: Vec<Duration> = (0..vec.len()).map(|i| Duration::from_secs(i as u64)).collect();
            let width = Duration::from_secs(width as u64);

            let mut min_wedge = MonoWedge::<u64>::min_wedge(width);
            let mut max_wedge = MonoWedge::<u64>::max_wedge(width);
            let mut ret_mins = Vec::with_capacity(expected_mins.len());
            let mut ret_maxs = Vec::with_capacity(expected_maxs.len());

            // Now we do the actual updates
            for (i, value) in time_points.iter().zip(&vec) {
                min_wedge.update((i, value));
                max_wedge.update((i, value));
                if i >= &(time_points[0] + width) {
                    ret_mins.push(min_wedge.front().unwrap_or_else(|| panic!("min_wedge should have at least 1 element")));
                    ret_maxs.push(max_wedge.front().unwrap_or_else(|| panic!("max_wedge should have at least 1 element")))
                }
            }

            let ret_mins: Vec<_> = ret_mins.into_iter().map(|s| s.1).copied().collect();
            let ret_maxs: Vec<_> = ret_maxs.into_iter().map(|s| s.1).copied().collect();
            assert_eq!(expected_mins, ret_mins, "window min incorrect");
            assert_eq!(expected_maxs, ret_maxs, "window max incorrect");
        }
    }
}
