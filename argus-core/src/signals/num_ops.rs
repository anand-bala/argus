use num_traits::{NumCast, Signed};

use super::traits::{LinearInterpolatable, SignalAbs};
use crate::signals::utils::{apply1, apply2};
use crate::signals::Signal;

impl<T> core::ops::Neg for &Signal<T>
where
    T: Signed + Copy,
{
    type Output = Signal<T>;

    /// Negate the signal at each time point
    fn neg(self) -> Self::Output {
        apply1(self, |v| -v)
    }
}

impl<T> core::ops::Add for &Signal<T>
where
    T: core::ops::Add<T, Output = T> + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs + rhs)
    }
}

impl<T> core::ops::Mul for &Signal<T>
where
    T: core::ops::Mul<T, Output = T> + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs * rhs)
    }
}

impl<T> core::ops::Sub for &Signal<T>
where
    T: core::ops::Sub<T, Output = T> + Copy + LinearInterpolatable + PartialOrd + NumCast,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: Self) -> Self::Output {
        use super::InterpolationMethod::Linear;
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.

        // If either of the signals are empty, we return an empty signal.
        if self.is_empty() || rhs.is_empty() {
            return Signal::new();
        }

        // the union of the sample points in self and other
        let sync_points = self.sync_with_intersection(rhs).unwrap();
        sync_points
            .into_iter()
            .map(|t| {
                let lhs = self.interpolate_at(t, Linear).unwrap();
                let rhs = rhs.interpolate_at(t, Linear).unwrap();
                (t, lhs - rhs)
            })
            .collect()
    }
}

impl<T> core::ops::Div for &Signal<T>
where
    T: core::ops::Div<T, Output = T> + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs / rhs)
    }
}

impl<T> num_traits::Pow<Self> for &Signal<T>
where
    T: num_traits::Pow<T, Output = T> + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Returns the values in `self` to the power of the values in `other`
    fn pow(self, other: Self) -> Self::Output {
        apply2(self, other, |lhs, rhs| lhs.pow(rhs))
    }
}

macro_rules! signal_abs_impl {
    ($( $ty:ty ), *) => {
        $(
        impl SignalAbs for Signal<$ty> {
            /// Return the absolute value for each sample in the signal
            fn abs(&self) -> Signal<$ty> {
                apply1(self, |v| v.abs())
            }
        }
        )*
    };
}

signal_abs_impl!(i64, f32, f64);

impl SignalAbs for Signal<u64> {
    /// Return the absolute value for each sample in the signal
    fn abs(&self) -> Signal<u64> {
        apply1(self, |v| v)
    }
}
