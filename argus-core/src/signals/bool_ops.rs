use super::interpolation::Linear;
use crate::signals::utils::{apply1, apply2};
use crate::signals::Signal;

impl core::ops::Not for &Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        apply1(self, |v| !v)
    }
}

impl core::ops::BitAnd<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: Self) -> Self::Output {
        apply2::<_, _, _, Linear>(self, other, |lhs, rhs| lhs && rhs)
    }
}

impl core::ops::BitOr<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: Self) -> Self::Output {
        apply2::<_, _, _, Linear>(self, other, |lhs, rhs| lhs || rhs)
    }
}
