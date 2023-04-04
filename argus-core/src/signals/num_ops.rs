use num_traits::{Num, NumCast, Signed};

use super::traits::{BaseSignal, LinearInterpolatable};
use crate::signals::utils::{apply1, apply2, apply2_const, sync_with_intersection};
use crate::signals::{ConstantSignal, Signal};

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

impl<T> core::ops::Neg for &ConstantSignal<T>
where
    T: Signed + Copy,
{
    type Output = ConstantSignal<T>;

    /// Negate the signal at each time point
    fn neg(self) -> Self::Output {
        ConstantSignal::new(self.value.neg())
    }
}

impl<T> core::ops::Add for &Signal<T>
where
    T: core::ops::Add<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs + rhs)
    }
}

impl<T> core::ops::Add for &ConstantSignal<T>
where
    T: core::ops::Add<T, Output = T> + Num + Copy,
{
    type Output = ConstantSignal<T>;

    /// Add the given signal with another
    fn add(self, rhs: Self) -> Self::Output {
        ConstantSignal::<T>::new(self.value + rhs.value)
    }
}

impl<T> core::ops::Add<&ConstantSignal<T>> for &Signal<T>
where
    T: core::ops::Add<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, rhs: &ConstantSignal<T>) -> Self::Output {
        apply2_const(self, rhs, |lhs, rhs| lhs + rhs)
    }
}

impl<T> core::ops::Add<&Signal<T>> for &ConstantSignal<T>
where
    T: core::ops::Add<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, rhs: &Signal<T>) -> Self::Output {
        rhs + self
    }
}

impl<T> core::ops::Mul for &Signal<T>
where
    T: core::ops::Mul<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs * rhs)
    }
}

impl<T> core::ops::Mul for &ConstantSignal<T>
where
    T: core::ops::Mul<T, Output = T> + Num + Copy,
{
    type Output = ConstantSignal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: Self) -> Self::Output {
        ConstantSignal::<T>::new(self.value * rhs.value)
    }
}

impl<T> core::ops::Mul<&ConstantSignal<T>> for &Signal<T>
where
    T: core::ops::Mul<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: &ConstantSignal<T>) -> Self::Output {
        apply2_const(self, rhs, |lhs, rhs| lhs * rhs)
    }
}

impl<T> core::ops::Mul<&Signal<T>> for &ConstantSignal<T>
where
    T: core::ops::Mul<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: &Signal<T>) -> Self::Output {
        rhs * self
    }
}

impl<T> core::ops::Sub for &Signal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable + PartialOrd,
    Signal<T>: BaseSignal<Value = T>,
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
        let sync_points = sync_with_intersection(self, rhs).unwrap();
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

impl<T> core::ops::Sub for &ConstantSignal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + Copy,
{
    type Output = ConstantSignal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: Self) -> Self::Output {
        ConstantSignal::<T>::new(self.value - rhs.value)
    }
}

impl<T> core::ops::Sub<&ConstantSignal<T>> for &Signal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable + PartialOrd,
    Signal<T>: BaseSignal<Value = T>,
    ConstantSignal<T>: BaseSignal<Value = T>,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: &ConstantSignal<T>) -> Self::Output {
        use super::InterpolationMethod::Linear;
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = sync_with_intersection(self, rhs).unwrap();
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

impl<T> core::ops::Sub<&Signal<T>> for &ConstantSignal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable + PartialOrd,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: &Signal<T>) -> Self::Output {
        use super::InterpolationMethod::Linear;
        // This has to be manually implemented and cannot use the apply2 functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        // the union of the sample points in self and other
        let sync_points = sync_with_intersection(self, rhs).unwrap();
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
    T: core::ops::Div<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs / rhs)
    }
}

impl<T> core::ops::Div for &ConstantSignal<T>
where
    T: core::ops::Div<T, Output = T> + Num + Copy,
{
    type Output = ConstantSignal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: Self) -> Self::Output {
        ConstantSignal::<T>::new(self.value / rhs.value)
    }
}

impl<T> core::ops::Div<&ConstantSignal<T>> for &Signal<T>
where
    T: core::ops::Div<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: &ConstantSignal<T>) -> Self::Output {
        apply2_const(self, rhs, |lhs, rhs| lhs / rhs)
    }
}

impl<T> core::ops::Div<&Signal<T>> for &ConstantSignal<T>
where
    T: core::ops::Div<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: &Signal<T>) -> Self::Output {
        apply2_const(rhs, self, |rhs, lhs| lhs / rhs)
    }
}

impl<T> num_traits::Pow<Self> for &Signal<T>
where
    T: num_traits::Pow<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Returns the values in `self` to the power of the values in `other`
    fn pow(self, other: Self) -> Self::Output {
        apply2(self, other, |lhs, rhs| lhs.pow(rhs))
    }
}
