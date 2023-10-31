//! A Rusty implementation of Alex Donze's adaptation[^1] of Daniel Lemire's streaming
//! min-max algorithm[^2] for piecewise linear signals.
//!
//! [^1]: Alexandre Donzé, Thomas Ferrère, and Oded Maler. 2013. Efficient Robust
//! Monitoring for STL. In Computer Aided Verification (Lecture Notes in Computer
//! Science), Springer, Berlin, Heidelberg, 264–279.
//!
//! [^2]: Daniel Lemire. 2007. Streaming Maximum-Minimum Filter Using No More than Three
//! Comparisons per Element. arXiv:cs/0610046.

// TODO: Make a MonoWedge iterator adapter.

use std::collections::{BTreeSet, VecDeque};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MonoWedge<T> {
    window: VecDeque<(Duration, T)>,
    pub(crate) time_points: BTreeSet<Duration>,
    cmp: fn(&T, &T) -> bool,
}

impl<T> MonoWedge<T> {
    pub fn new(cmp: fn(&T, &T) -> bool) -> Self {
        Self {
            window: Default::default(),
            time_points: Default::default(),
            cmp,
        }
    }
}

impl<T> MonoWedge<T> {
    pub fn update(&mut self, sample: (Duration, T)) {
        assert!(
            self.window.back().map_or(true, |v| v.0 < sample.0),
            "MonoWedge window samples don't have monotonic time"
        );
        // Find the index to partition the inner queue based on the comparison function.
        let cmp_idx = self.window.partition_point(|a| (self.cmp)(&a.1, &sample.1));
        assert!(cmp_idx <= self.window.len());

        // And delete all items in the second partition.
        let _ = self.window.split_off(cmp_idx);

        // Add new time point
        self.time_points.insert(sample.0);
        // Add the new value
        self.window.push_back(sample);
    }

    pub fn front(&self) -> Option<(&Duration, &T)> {
        Some((self.time_points.first()?, &self.window.front()?.1))
    }

    fn remove_older_samples(&mut self, t: Duration) {
        while let Some(item) = self.window.front() {
            if item.0 < t {
                let _ = self.window.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn purge_before(&mut self, t: Duration) {
        self.time_points = self.time_points.split_off(&t);
        self.remove_older_samples(t);
    }
}

impl<T> MonoWedge<T>
where
    T: PartialOrd,
{
    #[allow(dead_code)]
    pub fn min_wedge() -> Self {
        Self::new(T::lt)
    }

    pub fn max_wedge() -> Self {
        Self::new(T::gt)
    }
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use proptest::prelude::*;

    use super::*;

    fn run_test_min_max<T>(values: Vec<T>, width: usize)
    where
        T: Copy + Clone + core::cmp::PartialOrd + Ord + std::fmt::Debug,
    {
        // NOTE: When we convert the width from usize to Duration, the window becomes inclusive.
        let expected_mins: Vec<T> = (0..values.len())
            .map(|i| {
                let j = usize::min(i + width + 1, values.len());
                values[i..j].iter().min().copied().unwrap()
            })
            .collect();
        assert_eq!(expected_mins.len(), values.len());
        let expected_maxs: Vec<T> = (0..values.len())
            .map(|i| {
                let j = usize::min(i + width + 1, values.len());
                values[i..j].iter().max().copied().unwrap()
            })
            .collect();
        assert_eq!(expected_maxs.len(), values.len());

        let time_points: Vec<Duration> = (0..values.len()).map(|i| Duration::from_millis(i as u64)).collect();
        let start_time: Duration = time_points.first().copied().unwrap();
        let width = Duration::from_millis(width as u64);

        let mut min_wedge = MonoWedge::<T>::min_wedge();
        let mut max_wedge = MonoWedge::<T>::max_wedge();
        let mut ret_mins = Vec::with_capacity(time_points.len());
        let mut ret_maxs = Vec::with_capacity(time_points.len());

        let mut j: usize = 0;

        // Now we do the actual updates
        for (i, value) in time_points.iter().zip(&values) {
            min_wedge.purge_before(i.saturating_sub(width));
            min_wedge.update((*i, *value));
            max_wedge.purge_before(i.saturating_sub(width));
            max_wedge.update((*i, *value));
            if width <= *i - start_time {
                ret_mins.push(
                    min_wedge
                        .front()
                        .map(|(&t, &v)| (t, v))
                        .unwrap_or_else(|| panic!("min_wedge should have at least 1 element")),
                );
                ret_maxs.push(
                    max_wedge
                        .front()
                        .map(|(&t, &v)| (t, v))
                        .unwrap_or_else(|| panic!("max_wedge should have at least 1 element")),
                );
                j += 1;
            }
        }
        assert_eq!(j, ret_mins.len());
        assert_eq!(j, ret_maxs.len());
        assert!(j <= time_points.len());
        for i in &time_points[j..] {
            min_wedge.purge_before(*i);
            let min = min_wedge
                .front()
                .map(|(&t, &v)| (t, v))
                .unwrap_or_else(|| panic!("min_wedge should have at least 1 element"));
            assert_eq!(min.0, *i);
            ret_mins.push(min);

            max_wedge.purge_before(*i);
            let max = max_wedge
                .front()
                .map(|(&t, &v)| (t, v))
                .unwrap_or_else(|| panic!("max_wedge should have at least 1 element"));
            assert_eq!(max.0, *i);
            ret_maxs.push(max);
        }
        assert_eq!(time_points, ret_mins.iter().map(|s| s.0).collect_vec());
        assert_eq!(time_points, ret_maxs.iter().map(|s| s.0).collect_vec());

        let ret_mins: Vec<_> = ret_mins.into_iter().map(|s| s.1).collect();
        let ret_maxs: Vec<_> = ret_maxs.into_iter().map(|s| s.1).collect();
        assert_eq!(expected_mins, ret_mins, "window min incorrect");
        assert_eq!(expected_maxs, ret_maxs, "window max incorrect");
    }

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
            run_test_min_max(vec, width)
        }
    }

    #[test]
    fn smoketest_1() {
        let vec: Vec<u64> = vec![
            14978539203261649134,
            16311637665202408393,
            14583675943388486036,
            1550360951880186785,
            14850777793052200443,
        ];
        let width: usize = 2;

        run_test_min_max(vec, width)
    }

    #[test]
    fn smoketest_2() {
        let vec: Vec<u64> = vec![0, 0, 0];
        let width: usize = 2;

        run_test_min_max(vec, width)
    }
}
