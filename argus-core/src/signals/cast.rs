use core::iter::zip;

use crate::signals::traits::{SignalNumCast, TrySignalCast};
use crate::signals::Signal;
use crate::{ArgusError, ArgusResult};

macro_rules! impl_bool_to_num {
    ($to:ty) => {
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
}

impl SignalNumCast for Signal<bool> {
    impl_bool_to_num!(i8);
    impl_bool_to_num!(i16);
    impl_bool_to_num!(i32);
    impl_bool_to_num!(i64);
    impl_bool_to_num!(u8);
    impl_bool_to_num!(u16);
    impl_bool_to_num!(u32);
    impl_bool_to_num!(u64);

    #[inline]
    fn to_f32(&self) -> Option<Signal<f32>> {
        match self {
            Signal::Empty => Some(Signal::Empty),
            Signal::Constant { value } => {
                let value: f32 = if *value { f32::INFINITY } else { f32::NEG_INFINITY };
                Some(Signal::Constant { value })
            }
            Signal::Sampled { values, time_points } => zip(time_points, values)
                .map(|(&t, &v)| {
                    let val = num_traits::cast::<_, f32>(v as i64)?;
                    Some((t, val))
                })
                .collect(),
        }
    }

    #[inline]
    fn to_f64(&self) -> Option<Signal<f64>> {
        match self {
            Signal::Empty => Some(Signal::Empty),
            Signal::Constant { value } => {
                let value: f64 = if *value { f64::INFINITY } else { f64::NEG_INFINITY };
                Some(Signal::Constant { value })
            }
            Signal::Sampled { values, time_points } => zip(time_points, values)
                .map(|(&t, &v)| {
                    let val = num_traits::cast::<_, f64>(v as i64)?;
                    Some((t, val))
                })
                .collect(),
        }
    }
}

macro_rules! impl_cast {
    ($from:ty, [$( $to:ty ),*]) => {
        paste::paste! {
            impl SignalNumCast for Signal<$from> {
                $(
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
                )*
            }

            $(
            impl TrySignalCast<Signal<$to>> for Signal<$from> {
                fn try_cast(&self) -> ArgusResult<Signal<$to>> {
                    self.[<to_ $to>]().ok_or(ArgusError::InvalidCast {
                        from: std::any::type_name::<$from>(),
                        to: std::any::type_name::<$to>(),
                    })
                }
            }
            )*
        }
    };

    ($from:ty) => {
        impl_cast!($from, [i8, i16, i32, i64, u8, u16, u32, u64, f32, f64]);
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
