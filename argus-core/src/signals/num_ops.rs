use super::interpolation::Linear;
use super::{InterpolationMethod, SignalAbs};
use crate::signals::Signal;

impl<T> core::ops::Neg for Signal<T>
where
    T: core::ops::Neg<Output = T>,
{
    type Output = Signal<T>;

    fn neg(self) -> Self::Output {
        use Signal::*;
        match self {
            Empty => Signal::Empty,
            Constant { value } => Signal::constant(value.neg()),
            Sampled { values, time_points } => time_points.into_iter().zip(values.into_iter().map(|v| -v)).collect(),
        }
    }
}

impl<T> core::ops::Neg for &Signal<T>
where
    for<'a> &'a T: core::ops::Neg<Output = T>,
{
    type Output = Signal<T>;

    fn neg(self) -> Self::Output {
        use Signal::*;
        match self {
            Empty => Signal::Empty,
            Constant { value } => Signal::constant(value.neg()),
            Sampled { values, time_points } => time_points
                .iter()
                .copied()
                .zip(values.iter().map(|v| v.neg()))
                .collect(),
        }
    }
}

impl<T> core::ops::Add<&Signal<T>> for Signal<T>
where
    T: Clone,
    for<'a, 'b> &'a T: core::ops::Add<&'b T, Output = T>,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, other: &Signal<T>) -> Signal<T> {
        self.binary_op::<_, _, Linear>(other, |lhs, rhs| lhs + rhs)
    }
}

impl<T> core::ops::Add<&Signal<T>> for &Signal<T>
where
    T: Clone,
    for<'a, 'b> &'a T: core::ops::Add<&'b T, Output = T>,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Add the given signal with another
    fn add(self, other: &Signal<T>) -> Signal<T> {
        self.binary_op::<_, _, Linear>(other, |lhs, rhs| lhs + rhs)
    }
}

impl<T> core::ops::Mul<&Signal<T>> for Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Mul<&'b T, Output = T>,
    T: Clone,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: &Signal<T>) -> Signal<T> {
        self.binary_op::<_, _, Linear>(rhs, |lhs, rhs| lhs * rhs)
    }
}

impl<T> core::ops::Mul<&Signal<T>> for &Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Mul<&'b T, Output = T>,
    T: Clone,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Multiply the given signal with another
    fn mul(self, rhs: &Signal<T>) -> Signal<T> {
        self.binary_op::<_, _, Linear>(rhs, |lhs, rhs| lhs * rhs)
    }
}

impl<T> core::ops::Sub<&Signal<T>> for &Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Sub<&'b T, Output = T>,
    T: Clone + PartialOrd,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, other: &Signal<T>) -> Signal<T> {
        // This has to be manually implemented and cannot use the binary_op functions.
        // This is because if we have two signals that cross each other, then there is
        // an intermediate point where the two signals are equal. This point must be
        // added to the signal appropriately.
        use Signal::*;
        match (self, other) {
            // If either of the signals are empty, we return an empty signal.
            (Empty, _) | (_, Empty) => Signal::Empty,
            (Constant { value: v1 }, Constant { value: v2 }) => Signal::constant(v1 - v2),
            (lhs, rhs) => {
                // the union of the sample points in self and other
                let sync_points = lhs.sync_with_intersection::<Linear>(rhs).unwrap();
                sync_points
                    .into_iter()
                    .map(|t| {
                        let lhs = lhs.interpolate_at::<Linear>(t).unwrap();
                        let rhs = rhs.interpolate_at::<Linear>(t).unwrap();
                        (t, &lhs - &rhs)
                    })
                    .collect()
            }
        }
    }
}

impl<T> core::ops::Sub<&Signal<T>> for Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Sub<&'b T, Output = T>,
    T: Clone + PartialOrd,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Subtract the given signal with another
    fn sub(self, other: &Signal<T>) -> Signal<T> {
        <&Self as core::ops::Sub>::sub(&self, other)
    }
}

impl<T> core::ops::Div<&Signal<T>> for Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Div<&'b T, Output = T>,
    T: Clone,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: &Signal<T>) -> Self {
        self.binary_op::<_, _, Linear>(rhs, |lhs, rhs| lhs / rhs)
    }
}

impl<T> core::ops::Div<&Signal<T>> for &Signal<T>
where
    for<'a, 'b> &'a T: core::ops::Div<&'b T, Output = T>,
    T: Clone,
    Linear: InterpolationMethod<T>,
{
    type Output = Signal<T>;

    /// Divide the given signal with another
    fn div(self, rhs: &Signal<T>) -> Signal<T> {
        self.binary_op::<_, _, Linear>(rhs, |lhs, rhs| lhs / rhs)
    }
}

impl<T> Signal<T>
where
    for<'a, 'b> &'a T: num_traits::Pow<&'b T, Output = T>,
    T: Clone,
    Linear: InterpolationMethod<T>,
{
    /// Returns the values in `self` to the power of the values in `other`
    pub fn pow(&self, other: &Self) -> Self {
        use num_traits::Pow;
        self.binary_op::<_, _, Linear>(other, |lhs, rhs| lhs.pow(rhs))
    }
}

macro_rules! signal_abs_impl {
    ($( $ty:ty ), *) => {
        $(
        impl SignalAbs for Signal<$ty> {
            /// Return the absolute value for each sample in the signal
            fn abs(self) -> Signal<$ty> {
                self.unary_op(|v| v.abs())
            }
        }
        )*
    };
}

signal_abs_impl!(i64, f32, f64);

impl SignalAbs for Signal<u64> {
    /// Return the absolute value for each sample in the signal
    fn abs(self) -> Signal<u64> {
        self.unary_op(|v| v)
    }
}
