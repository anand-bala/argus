use std::cmp::Ordering;
use std::time::Duration;

use paste::paste;

use super::{Sample, Signal};

/// Trait for values that are linear interpolatable
pub trait LinearInterpolatable {
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized;
}

impl LinearInterpolatable for bool {
    fn interpolate_at(a: &Sample<Self>, b: &Sample<Self>, time: Duration) -> Self
    where
        Self: Sized,
    {
        assert!(a.time < time && time < b.time);
        // We can't linear interpolate a boolean, so we return the previous.
        a.value
    }
}

macro_rules! interpolate_for_num {
    ($ty:ty) => {
        impl LinearInterpolatable for $ty {
            fn interpolate_at(first: &Sample<Self>, second: &Sample<Self>, time: Duration) -> Self
            where
                Self: Sized,
            {
                use num_traits::cast;
                // We will need to cast the samples to f64 values (along with the time
                // window) to be able to interpolate correctly.
                // TODO(anand): Verify this works.
                let t1 = first.time.as_secs_f64();
                let t2 = second.time.as_secs_f64();
                let at = time.as_secs_f64();
                assert!((t1..=t2).contains(&at));

                // We need to do stable linear interpolation
                // https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2019/p0811r3.html
                let a: f64 = cast(first.value).unwrap();
                let b: f64 = cast(second.value).unwrap();

                // Set t to a value in [0, 1]
                let t = (at - t1) / (t2 - t1);
                assert!((0.0..=1.0).contains(&t));

                let val = if (a <= 0.0 && b >= 0.0) || (a >= 0.0 && b <= 0.0) {
                    t * b + (1.0 - t) * a
                } else if t == 1.0 {
                    b
                } else {
                    a + t * (b - a)
                };

                cast(val).unwrap()
            }
        }
    };
}

interpolate_for_num!(i8);
interpolate_for_num!(i16);
interpolate_for_num!(i32);
interpolate_for_num!(i64);
interpolate_for_num!(u8);
interpolate_for_num!(u16);
interpolate_for_num!(u32);
interpolate_for_num!(u64);
interpolate_for_num!(f32);
interpolate_for_num!(f64);

macro_rules! impl_signal_cmp {
    ($cmp:ident) => {
        paste! {
            fn [<signal_ $cmp>](&self, other: &Rhs) -> Option<Signal<bool>> {
                self.signal_cmp(other, |ord| ord.[<is_ $cmp>]())
            }
        }
    };
}

/// A time-wise partial ordering defined for signals
pub trait SignalPartialOrd<Rhs = Self> {
    /// Compare two signals within each of their domains (using [`PartialOrd`]) and
    /// apply the given function `op` to the ordering to create a signal.
    ///
    /// This function returns `None` if the comparison isn't possible, namely, when
    /// either of the signals are empty.
    fn signal_cmp<F>(&self, other: &Rhs, op: F) -> Option<Signal<bool>>
    where
        F: Fn(Ordering) -> bool;

    impl_signal_cmp!(lt);
    impl_signal_cmp!(le);
    impl_signal_cmp!(gt);
    impl_signal_cmp!(ge);
    impl_signal_cmp!(eq);
    impl_signal_cmp!(ne);
}

/// Time-wise min-max of signal types
pub trait SignalMinMax<Rhs = Self> {
    type Output;

    /// Compute the time-wise min of two signals
    fn min(&self, rhs: &Rhs) -> Self::Output;

    /// Compute the time-wise max of two signals
    fn max(&self, rhs: &Rhs) -> Self::Output;
}

/// Trait for converting between numeric signal types
pub trait SignalNumCast {
    fn to_i8(&self) -> Option<Signal<i8>>;
    fn to_i16(&self) -> Option<Signal<i16>>;
    fn to_i32(&self) -> Option<Signal<i32>>;
    fn to_i64(&self) -> Option<Signal<i64>>;
    fn to_u8(&self) -> Option<Signal<u8>>;
    fn to_u16(&self) -> Option<Signal<u16>>;
    fn to_u32(&self) -> Option<Signal<u32>>;
    fn to_u64(&self) -> Option<Signal<u64>>;
    fn to_f32(&self) -> Option<Signal<f32>>;
    fn to_f64(&self) -> Option<Signal<f64>>;
}

/// Trait for computing the absolute value of the samples in a signal
pub trait SignalAbs {
    fn abs(&self) -> Self;
}
