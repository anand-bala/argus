use core::iter::zip;

use crate::signals::traits::SignalNumCast;
use crate::signals::Signal;

macro_rules! impl_cast {
    (bool => $to:ty) => {
        paste::paste! {
            #[inline]
            fn [<to_ $to>](&self) -> Option<Signal<$to>> {
                match self {
                    Signal::Empty => Some(Signal::Empty),
                    Signal::Constant { value } => num_traits::cast::<_, $to>(*value as i64).map(Signal::constant),
                    Signal::Sampled { values, time_points } => {
                        zip(time_points, values)
                        .map(|(&t, &v)| {
                        let val = num_traits::cast::<_, $to>(v as i64)?;
                            Some((t, val))
                        })
                        .collect()
                    }
                }
            }
        }
    };
    ($from:ty => $to:ty) => {
        paste::paste! {
            #[inline]
            fn [<to_ $to>](&self) -> Option<Signal<$to>> {
                match self {
                    Signal::Empty => Some(Signal::Empty),
                    Signal::Constant { value } => num_traits::cast::<_, $to>(*value).map(Signal::constant),
                    Signal::Sampled { values, time_points } => {
                        zip(time_points, values)
                        .map(|(&t, &v)| {
                        let val = num_traits::cast::<_, $to>(v)?;
                            Some((t, val))
                        })
                        .collect()
                    }
                }
            }
        }
    };

    ($from:ty) => {
        impl SignalNumCast for Signal<$from> {
            impl_cast!($from => i8);
            impl_cast!($from => i16);
            impl_cast!($from => i32);
            impl_cast!($from => i64);
            impl_cast!($from => u8);
            impl_cast!($from => u16);
            impl_cast!($from => u32);
            impl_cast!($from => u64);
            impl_cast!($from => f32);
            impl_cast!($from => f64);
        }
    };
}

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
