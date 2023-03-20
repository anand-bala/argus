use std::ops::{BitAnd, BitOr, Not};

use super::{internal_macros, BoolExpr};

impl Not for BoolExpr {
    type Output = BoolExpr;

    fn not(self) -> Self::Output {
        BoolExpr::Not { arg: Box::new(self) }
    }
}

impl Not for Box<BoolExpr> {
    type Output = BoolExpr;

    fn not(self) -> Self::Output {
        BoolExpr::Not { arg: self }
    }
}

impl BitOr for BoolExpr {
    type Output = BoolExpr;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        use BoolExpr::*;

        match (self, rhs) {
            (Or { args: mut left }, Or { args: mut right }) => {
                left.append(&mut right);
                Or { args: left }
            }
            (Or { mut args }, other) | (other, Or { mut args }) => {
                args.push(other);
                Or { args }
            }
            (left, right) => {
                let args = vec![left, right];
                Or { args }
            }
        }
    }
}

internal_macros::forward_box_binop! {impl BitOr, bitor for BoolExpr, BoolExpr }

impl BitAnd for BoolExpr {
    type Output = BoolExpr;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        use BoolExpr::*;

        match (self, rhs) {
            (And { args: mut left }, And { args: mut right }) => {
                left.append(&mut right);
                And { args: left }
            }
            (And { mut args }, other) | (other, And { mut args }) => {
                args.push(other);
                And { args }
            }
            (left, right) => {
                let args = vec![left, right];
                And { args }
            }
        }
    }
}

internal_macros::forward_box_binop! {impl BitAnd, bitand for BoolExpr, BoolExpr }
