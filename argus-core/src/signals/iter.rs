//! Signal iterator.

use std::iter::{zip, Zip};
use std::time::Duration;

use super::Signal;

/// An iterator over a [`Signal`].
///
/// This takes into account if the signal is iterable or not, i.e., it produces samples
/// only for [`Signal::Sampled`] and empty iterators for [`Signal::Empty`] (as it
/// contains no values) and [`Signal::Constant`] (as there is no well defined start and
/// end to the signal).
#[derive(Debug, Default)]
pub enum Iter<'a, T> {
    #[doc(hidden)]
    #[default]
    Empty,
    #[doc(hidden)]
    Iter(Zip<core::slice::Iter<'a, Duration>, core::slice::Iter<'a, T>>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a Duration, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty => None,
            Iter::Iter(iter) => iter.next(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Signal<T> {
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Signal::Empty => Iter::default(),
            Signal::Constant { value: _ } => Iter::default(),
            Signal::Sampled { values, time_points } => Iter::Iter(zip(time_points, values)),
        }
    }
}

impl<T> FromIterator<(Duration, T)> for Signal<T> {
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
