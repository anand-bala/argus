use super::interpolation::Linear;
use crate::signals::utils::{apply1, apply2};
use crate::signals::Signal;

impl core::ops::Not for &Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        apply1(self, |v| !v)
    }
}

impl Signal<bool> {
    /// Apply logical conjunction for each sample across the two signals.
    ///
    /// Here, the conjunction is taken at all signal points where either of the signals
    /// are sampled, and where they intersect (using interpolation).
    ///
    /// See [`Signal::sync_with_intersection`].
    pub fn and(&self, other: &Self) -> Self {
        apply2::<_, _, _, Linear>(self, other, |lhs, rhs| lhs && rhs)
    }

    /// Apply logical disjunction for each sample across the two signals.
    ///
    /// Here, the disjunction is taken at all signal points where either of the signals
    /// are sampled, and where they intersect (using interpolation).
    ///
    /// See [`Signal::sync_with_intersection`].
    pub fn or(&self, other: &Self) -> Self {
        apply2::<_, _, _, Linear>(self, other, |lhs, rhs| lhs || rhs)
    }
}

impl core::ops::BitAnd<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: Self) -> Self::Output {
        self.and(other)
    }
}

impl core::ops::BitOr<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: Self) -> Self::Output {
        self.or(other)
    }
}
