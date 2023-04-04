use itertools::Itertools;
use num_traits::{Num, NumCast};

use crate::signals::traits::SignalNumCast;
use crate::signals::{AnySignal, ConstantSignal, Signal};

macro_rules! impl_cast {
    ($type:ty) => {
        paste::paste! {
            #[inline]
            fn [<to_ $type>](&self) -> Option<Signal<$type>> {
                let samples = self
                    .iter()
                    .map_while(|(&t, &v)| num_traits::cast::<_, $type>(v).map(|v| (t, v)))
                    .collect_vec();
                if samples.len() < self.time_points.len() {
                    // Failed to convert some item
                    None
                } else {
                    Some(samples.into_iter().collect())
                }
            }
        }
    };
}

impl<T> SignalNumCast for Signal<T>
where
    T: Num + NumCast + Copy,
{
    type Value = T;

    type Output<U> = Signal<U>
    where
        U: Num + NumCast + Copy;

    impl_cast!(i8);
    impl_cast!(i16);
    impl_cast!(i32);
    impl_cast!(i64);
    impl_cast!(u8);
    impl_cast!(u16);
    impl_cast!(u32);
    impl_cast!(u64);
    impl_cast!(f32);
    impl_cast!(f64);
}

macro_rules! impl_cast {
    ($type:ty) => {
        paste::paste! {
            #[inline]
            fn [<to_ $type>](&self) -> Option<ConstantSignal<$type>> {
                num_traits::cast::<_, $type>(self.value).map(ConstantSignal::new)
            }
        }
    };
}

impl<T> SignalNumCast for ConstantSignal<T>
where
    T: Num + NumCast + Copy,
{
    type Value = T;

    type Output<U> = ConstantSignal<U>
    where
        U: Num + NumCast + Copy;

    impl_cast!(i8);
    impl_cast!(i16);
    impl_cast!(i32);
    impl_cast!(i64);
    impl_cast!(u8);
    impl_cast!(u16);
    impl_cast!(u32);
    impl_cast!(u64);
    impl_cast!(f32);
    impl_cast!(f64);
}
