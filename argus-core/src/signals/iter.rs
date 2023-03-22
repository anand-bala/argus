use std::iter::Zip;
use std::time::Duration;

use super::Signal;

pub struct Iter<'a, T> {
    iter: Zip<core::slice::Iter<'a, Duration>, core::slice::Iter<'a, T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a Duration, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T> IntoIterator for &'a Signal<T> {
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.time_points.iter().zip(self.values.iter()),
        }
    }
}

impl<T> FromIterator<(Duration, T)> for Signal<T>
where
    T: Copy,
{
    /// Takes a sequence of sample points and creates a signal.
    ///
    /// # Panics
    ///
    /// If the input data does not contain strictly monotonically increasing time
    /// stamps. If this isn't desired, sort and deduplicate the input data.
    fn from_iter<I: IntoIterator<Item = (Duration, T)>>(iter: I) -> Self {
        Self::try_from_iter(iter).unwrap()
    }
}
