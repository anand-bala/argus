use core::iter::zip;

use crate::signals::traits::{SignalNumCast, TrySignalCast};
use crate::signals::Signal;
use crate::{ArgusError, ArgusResult};

macro_rules! impl_cast_bool {
    ([$( $to:ty ),*]) => {
        paste::paste! {
            impl SignalNumCast for Signal<bool> {
                $(
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
                )*
            }

            $(
            impl TrySignalCast<Signal<$to>> for Signal<bool> {
                fn try_cast(&self) -> ArgusResult<Signal<$to>> {
                    self.[<to_ $to>]().ok_or(ArgusError::InvalidCast {
                        from: std::any::type_name::<bool>(),
                        to: std::any::type_name::<$to>(),
                    })
                }
            }
            )*
        }
    };
    () => {
        impl_cast_bool!([i8, i16, i32, i64, u8, u16, u32, u64, f32, f64]);
    };
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

impl_cast_bool!();
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
