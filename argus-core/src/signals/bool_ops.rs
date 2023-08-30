use super::interpolation::Linear;
use crate::signals::Signal;

impl core::ops::Not for Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        use Signal::*;
        match self {
            Empty => self,
            Constant { value } => Signal::constant(!value),
            signal => signal.into_iter().map(|(&t, v)| (t, !v)).collect(),
        }
    }
}

impl core::ops::Not for &Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        use Signal::*;
        match self {
            Empty => Empty,
            Constant { value } => Signal::constant(!value),
            signal => signal.into_iter().map(|(&t, &v)| (t, !v)).collect(),
        }
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
        self.binary_op::<_, _, Linear>(other, |lhs, rhs| *lhs && *rhs)
    }

    /// Apply logical disjunction for each sample across the two signals.
    ///
    /// Here, the disjunction is taken at all signal points where either of the signals
    /// are sampled, and where they intersect (using interpolation).
    ///
    /// See [`Signal::sync_with_intersection`].
    pub fn or(&self, other: &Self) -> Self {
        self.binary_op::<_, _, Linear>(other, |lhs, rhs| *lhs || *rhs)
    }
}

impl core::ops::BitAnd<&Signal<bool>> for Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: &Signal<bool>) -> Self::Output {
        self.and(other)
    }
}

impl core::ops::BitAnd<&Signal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: &Signal<bool>) -> Self::Output {
        self.and(other)
    }
}

impl core::ops::BitOr<&Signal<bool>> for Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: &Signal<bool>) -> Self::Output {
        self.or(other)
    }
}

impl core::ops::BitOr<&Signal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: &Signal<bool>) -> Self::Output {
        self.or(other)
    }
}
