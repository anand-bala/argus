use num_traits::{Num, NumCast, Signed};

use crate::signals::utils::{apply1, apply2, apply2_const};
use crate::signals::{ConstantSignal, Signal};

use super::traits::LinearInterpolatable;

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
    T: core::ops::Add<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs + rhs)
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

impl<T> core::ops::Sub for &Signal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: Self) -> Self::Output {
        apply2(self, rhs, |lhs, rhs| lhs - rhs)
    }
}

impl<T> core::ops::Sub<&ConstantSignal<T>> for &Signal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Subtiply the given signal with another
    fn sub(self, rhs: &ConstantSignal<T>) -> Self::Output {
        apply2_const(self, rhs, |lhs, rhs| lhs - rhs)
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
impl<T> core::ops::Sub<&Signal<T>> for &ConstantSignal<T>
where
    T: core::ops::Sub<T, Output = T> + Num + NumCast + Copy + LinearInterpolatable,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, rhs: &Signal<T>) -> Self::Output {
        apply2_const(rhs, self, |rhs, lhs| lhs - rhs)
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
