use std::ops::{Add, Div, Mul, Neg};

use super::{internal_macros, BoolExpr, NumExpr};

impl Neg for NumExpr {
    type Output = NumExpr;

    #[inline]
    fn neg(self) -> Self::Output {
        NumExpr::Neg { arg: Box::new(self) }
    }
}

impl Neg for Box<NumExpr> {
    type Output = NumExpr;

    #[inline]
    fn neg(self) -> Self::Output {
        NumExpr::Neg { arg: self }
    }
}

impl Add for NumExpr {
    type Output = NumExpr;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        use NumExpr::*;

        match (self, rhs) {
            (Add { args: mut left }, Add { args: mut right }) => {
                left.append(&mut right);
                Add { args: left }
            }
            (Add { mut args }, other) | (other, Add { mut args }) => {
                args.push(other);
                Add { args }
            }
            (left, right) => {
                let args = vec![left, right];
                Add { args }
            }
        }
    }
}

internal_macros::forward_box_binop! {impl Add, add for NumExpr, NumExpr }

impl Mul for NumExpr {
    type Output = NumExpr;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        use NumExpr::*;

        match (self, rhs) {
            (Mul { args: mut left }, Mul { args: mut right }) => {
                left.append(&mut right);
                Mul { args: left }
            }
            (Mul { mut args }, other) | (other, Mul { mut args }) => {
                args.push(other);
                Mul { args }
            }
            (left, right) => {
                let args = vec![left, right];
                Mul { args }
            }
        }
    }
}

internal_macros::forward_box_binop! {impl Mul, mul for NumExpr, NumExpr }

impl Div for NumExpr {
    type Output = NumExpr;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        use NumExpr::*;
        Div {
            dividend: Box::new(self),
            divisor: Box::new(rhs),
        }
    }
}

internal_macros::forward_box_binop! {impl Div, div for NumExpr, NumExpr }

use super::Ordering;

impl NumExpr {
    pub fn less_than(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::Less { strict: true },
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }

    pub fn less_than_eq(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::Less { strict: false },
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }

    pub fn greater_than(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::Greater { strict: true },
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }

    pub fn greater_than_eq(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::Greater { strict: false },
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }

    pub fn equal(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::Eq,
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }

    pub fn not_equal(self, rhs: Self) -> BoolExpr {
        BoolExpr::Cmp {
            op: Ordering::NotEq,
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }
}
