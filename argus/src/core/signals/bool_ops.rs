use super::interpolation::Linear;
use super::InterpolationMethod;
use crate::signals::Signal;

impl core::ops::Not for Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        self.logical_not()
    }
}

impl core::ops::Not for &Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        self.logical_not()
    }
}

impl Signal<bool> {
    /// Apply logical not for each sample across the signal.
    pub fn logical_not(&self) -> Self {
        self.unary_op(|&v| !v)
    }

    /// Apply logical conjunction for each sample across the two signals.
    ///
    /// Here, the conjunction is taken at all signal points where either of the signals
    /// are sampled, and where they intersect (using interpolation).
    ///
    /// See [`Signal::sync_with_intersection`].
    pub fn and<I: InterpolationMethod<bool>>(&self, other: &Self) -> Self {
        self.binary_op::<_, _, I>(other, |lhs, rhs| *lhs && *rhs)
    }

    /// Apply logical disjunction for each sample across the two signals.
    ///
    /// Here, the disjunction is taken at all signal points where either of the signals
    /// are sampled, and where they intersect (using interpolation).
    ///
    /// See [`Signal::sync_with_intersection`].
    pub fn or<I: InterpolationMethod<bool>>(&self, other: &Self) -> Self {
        self.binary_op::<_, _, I>(other, |lhs, rhs| *lhs || *rhs)
    }
}

impl core::ops::BitAnd<&Signal<bool>> for Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: &Signal<bool>) -> Self::Output {
        self.and::<Linear>(other)
    }
}

impl core::ops::BitAnd<&Signal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: &Signal<bool>) -> Self::Output {
        self.and::<Linear>(other)
    }
}

impl core::ops::BitOr<&Signal<bool>> for Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: &Signal<bool>) -> Self::Output {
        self.or::<Linear>(other)
    }
}

impl core::ops::BitOr<&Signal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: &Signal<bool>) -> Self::Output {
        self.or::<Linear>(other)
    }
}
