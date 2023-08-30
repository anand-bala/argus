use core::iter::zip;

use crate::signals::Signal;
use crate::{ArgusError, ArgusResult};

impl<T> Signal<T>
where
    T: num_traits::NumCast + Copy,
{
    /// Cast a numeric signal to another numeric signal
    pub fn num_cast<U>(&self) -> ArgusResult<Signal<U>>
    where
        U: num_traits::NumCast,
    {
        let ret: Option<_> = match self {
            Signal::Empty => Some(Signal::Empty),
            Signal::Constant { value } => num_traits::cast(*value).map(Signal::constant),
            Signal::Sampled { values, time_points } => zip(time_points, values)
                .map(|(&t, &v)| {
                    let val: U = num_traits::cast(v)?;
                    Some((t, val))
                })
                .collect(),
        };

        ret.ok_or(ArgusError::invalid_cast::<T, U>())
    }
}
