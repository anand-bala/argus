use crate::signals::utils::{apply1, apply2, apply2_const};
use crate::signals::{ConstantSignal, Signal};

impl core::ops::Not for &Signal<bool> {
    type Output = Signal<bool>;

    fn not(self) -> Self::Output {
        apply1(self, |v| !v)
    }
}

impl core::ops::BitAnd<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: Self) -> Self::Output {
        apply2(self, other, |lhs, rhs| lhs && rhs)
    }
}

impl core::ops::BitAnd<&ConstantSignal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, other: &ConstantSignal<bool>) -> Self::Output {
        apply2_const(self, other, |lhs, rhs| lhs && rhs)
    }
}

impl core::ops::BitOr<Self> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: Self) -> Self::Output {
        apply2(self, other, |lhs, rhs| lhs || rhs)
    }
}

impl core::ops::BitOr<&ConstantSignal<bool>> for &Signal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, other: &ConstantSignal<bool>) -> Self::Output {
        apply2_const(self, other, |lhs, rhs| lhs || rhs)
    }
}

impl core::ops::Not for &ConstantSignal<bool> {
    type Output = ConstantSignal<bool>;

    fn not(self) -> Self::Output {
        ConstantSignal::<bool>::new(!self.value)
    }
}

impl core::ops::BitAnd<Self> for &ConstantSignal<bool> {
    type Output = ConstantSignal<bool>;

    fn bitand(self, rhs: Self) -> Self::Output {
        ConstantSignal::<bool>::new(self.value && rhs.value)
    }
}

impl core::ops::BitAnd<&Signal<bool>> for &ConstantSignal<bool> {
    type Output = Signal<bool>;

    fn bitand(self, rhs: &Signal<bool>) -> Self::Output {
        rhs & self
    }
}
impl core::ops::BitOr<Self> for &ConstantSignal<bool> {
    type Output = ConstantSignal<bool>;

    fn bitor(self, rhs: Self) -> Self::Output {
        ConstantSignal::<bool>::new(self.value || rhs.value)
    }
}

impl core::ops::BitOr<&Signal<bool>> for &ConstantSignal<bool> {
    type Output = Signal<bool>;

    fn bitor(self, rhs: &Signal<bool>) -> Self::Output {
        rhs | self
    }
}
