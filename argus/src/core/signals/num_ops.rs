use super::{InterpolationMethod, SignalAbs};
use crate::signals::Signal;

impl<T> Signal<T> {
    /// Perform sample-wise arithmetic negation over the signal.
    pub fn negate<U>(&self) -> Signal<U>
    where
        for<'a> &'a T: core::ops::Neg<Output = U>,
    {
        self.unary_op(|v| -v)
    }

    /// Perform sample-wise addition of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn add<U, I>(&self, other: &Signal<T>) -> Signal<U>
    where
        for<'t> &'t T: core::ops::Add<Output = U>,
        T: Clone,
        I: InterpolationMethod<T>,
    {
        self.binary_op::<_, _, I>(other, |u, v| u + v)
    }

    /// Perform sample-wise multiplication of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn mul<U, I>(&self, other: &Signal<T>) -> Signal<U>
    where
        for<'t> &'t T: core::ops::Mul<Output = U>,
        T: Clone,
        I: InterpolationMethod<T>,
    {
        self.binary_op::<_, _, I>(other, |u, v| u * v)
    }

    /// Perform sample-wise subtraction of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn sub<U, I>(&self, other: &Signal<T>) -> Signal<U>
    where
        for<'t> &'t T: core::ops::Sub<Output = U>,
        T: Clone + PartialOrd,
        I: InterpolationMethod<T>,
    {
        self.binary_op_with_intersection::<_, _, I>(other, |u, v| u - v)
    }

    /// Perform sample-wise division of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn div<U, I>(&self, other: &Signal<T>) -> Signal<U>
    where
        for<'t> &'t T: core::ops::Div<Output = U>,
        T: Clone,
        I: InterpolationMethod<T>,
    {
        self.binary_op::<_, _, I>(other, |u, v| u / v)
    }

    /// Perform sample-wise exponentiation of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn pow<U, I>(&self, exponent: &Signal<T>) -> Signal<U>
    where
        for<'a, 'b> &'a T: num_traits::Pow<&'b T, Output = U>,
        T: Clone,
        I: InterpolationMethod<T>,
    {
        use num_traits::Pow;
        self.binary_op::<_, _, I>(exponent, |u, v| u.pow(v))
    }

    /// Perform sample-wise absolute difference of the two signals.
    ///
    /// Here, a new point is computed for all signal points where either of the signals
    /// are sampled and where they intersect (using interpolation).
    pub fn abs_diff<U, I>(&self, other: &Signal<T>) -> Signal<U>
    where
        for<'t> &'t T: core::ops::Sub<Output = U>,
        T: Clone + PartialOrd,
        I: InterpolationMethod<T>,
    {
        self.binary_op_with_intersection::<_, _, I>(other, |u, v| if u < v { v - u } else { u - v })
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
        self.unary_op(|&v| v)
    }
}
