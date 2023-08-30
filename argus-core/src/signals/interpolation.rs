//! Interpolation methods

use std::time::Duration;

use super::utils::Neighborhood;
use super::{FindIntersectionMethod, InterpolationMethod, Sample};

/// Constant interpolation.
///
/// Here, the previous signal value is propagated to the requested time point.
pub struct Constant;

impl<T: Clone> InterpolationMethod<T> for Constant {
    fn at(a: &Sample<T>, b: &Sample<T>, time: Duration) -> Option<T> {
        if time == b.time {
            Some(b.value.clone())
        } else if a.time <= time && time < b.time {
            Some(a.value.clone())
        } else {
            None
        }
    }
}

/// Nearest interpolation.
///
/// Here, the signal value from the nearest sample (time-wise) is propagated to the
/// requested time point.
pub struct Nearest;

impl<T: Clone> InterpolationMethod<T> for Nearest {
    fn at(a: &super::Sample<T>, b: &super::Sample<T>, time: std::time::Duration) -> Option<T> {
        if time < a.time || time > b.time {
            // `time` is outside the segments.
            None
        } else if (b.time - time) > (time - a.time) {
            // a is closer to the required time than b
            Some(a.value.clone())
        } else {
            // b is closer
            Some(b.value.clone())
        }
    }
}

/// Linear interpolation.
///
/// Here, linear interpolation is performed to estimate the sample at the time point
/// between two samples.
pub struct Linear;

impl InterpolationMethod<bool> for Linear {
    fn at(a: &Sample<bool>, b: &Sample<bool>, time: Duration) -> Option<bool> {
        if a.time < time && time < b.time {
            // We can't linear interpolate a boolean, so we return the previous.
            Some(a.value)
        } else {
            None
        }
    }
}

impl FindIntersectionMethod<bool> for Linear {
    fn find_intersection(a: &Neighborhood<bool>, b: &Neighborhood<bool>) -> Sample<bool> {
        let Sample { time: ta1, value: ya1 } = a.first.unwrap();
        let Sample { time: ta2, value: ya2 } = a.second.unwrap();
        let Sample { time: tb1, value: yb1 } = b.first.unwrap();
        let Sample { time: tb2, value: yb2 } = b.second.unwrap();

        let left_cmp = ya1.cmp(&yb1);
        let right_cmp = ya2.cmp(&yb2);

        if left_cmp.is_eq() {
            // They already intersect, so we return the inner time-point
            if ta1 < tb1 {
                Sample { time: tb1, value: yb1 }
            } else {
                Sample { time: ta1, value: ya1 }
            }
        } else if right_cmp.is_eq() {
            // They intersect at the end, so we return the outer time-point, as that is
            // when they become equal.
            if ta2 < tb2 {
                Sample { time: tb2, value: yb2 }
            } else {
                Sample { time: ta2, value: ya2 }
            }
        } else {
            // The switched, so the one that switched earlier will intersect with the
            // other.
            // So, we find the one that has a lower time point, i.e., the inner one.
            if ta2 < tb2 {
                Sample { time: ta2, value: ya2 }
            } else {
                Sample { time: tb2, value: yb2 }
            }
        }
    }
}

macro_rules! interpolate_for_num {
    ($ty:ty) => {
        impl InterpolationMethod<$ty> for Linear {
            fn at(first: &Sample<$ty>, second: &Sample<$ty>, time: Duration) -> Option<$ty> {
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
                let a: f64 = cast(first.value).unwrap_or_else(|| panic!("unable to cast {:?} to f64", first.value));
                let b: f64 = cast(second.value).unwrap_or_else(|| panic!("unable to cast {:?} to f64", second.value));

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

                cast(val)
            }
        }

        impl FindIntersectionMethod<$ty> for Linear {
            fn find_intersection(a: &Neighborhood<$ty>, b: &Neighborhood<$ty>) -> Sample<$ty> {
                // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line
                use num_traits::cast;

                let Sample { time: t1, value: y1 } = (a.first).unwrap();
                let Sample { time: t2, value: y2 } = (a.second).unwrap();
                let Sample { time: t3, value: y3 } = (b.first).unwrap();
                let Sample { time: t4, value: y4 } = (b.second).unwrap();

                let t1 = t1.as_secs_f64();
                let t2 = t2.as_secs_f64();
                let t3 = t3.as_secs_f64();
                let t4 = t4.as_secs_f64();

                let y1: f64 = cast(y1).unwrap_or_else(|| panic!("unable to cast {:?} to f64", y1));
                let y2: f64 = cast(y2).unwrap_or_else(|| panic!("unable to cast {:?} to f64", y2));
                let y3: f64 = cast(y3).unwrap_or_else(|| panic!("unable to cast {:?} to f64", y3));
                let y4: f64 = cast(y4).unwrap_or_else(|| panic!("unable to cast {:?} to f64", y4));

                let denom = ((t1 - t2) * (y3 - y4)) - ((y1 - y2) * (t3 - t4));

                let t_top = (((t1 * y2) - (y1 * t2)) * (t3 - t4)) - ((t1 - t2) * (t3 * y4 - y3 * t4));
                let y_top = (((t1 * y2) - (y1 * t2)) * (y3 - y4)) - ((y1 - y2) * (t3 * y4 - y3 * t4));

                let t = Duration::from_secs_f64(t_top / denom);
                let y: $ty = num_traits::cast(y_top / denom).unwrap();
                Sample { time: t, value: y }
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
