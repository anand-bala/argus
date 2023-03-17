use std::collections::HashSet;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not};

pub(crate) mod internal_macros;

use crate::{ArgusResult, Error};

/// All expressions that are numeric
#[derive(Clone, Debug)]
pub enum NumExpr {
    IntLit(i64),
    UIntLit(u64),
    FloatLit(f64),
    IntVar { name: String },
    UIntVar { name: String },
    FloatVar { name: String },

    Neg { arg: Box<NumExpr> },
    Add { args: Vec<NumExpr> },
    Mul { args: Vec<NumExpr> },
    Div { dividend: Box<NumExpr>, divisor: Box<NumExpr> },
}

/// Types of comparison operations
#[derive(Clone, Copy, Debug)]
pub enum Ordering {
    Eq,
    NotEq,
    Less { strict: bool },
    Greater { strict: bool },
}

/// All expressions that are evaluated to be of type `bool`
#[derive(Clone, Debug)]
pub enum BoolExpr {
    BoolLit(bool),
    BoolVar { name: String },
    Cmp { op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr> },
    Not { arg: Box<BoolExpr> },
    And { args: Vec<BoolExpr> },
    Or { args: Vec<BoolExpr> },
}

/// Expression builder
///
/// The `ExprBuilder` is a factory structure that deals with the creation of
/// expressions. The main goal of this is to ensure users do not create duplicate
/// definitions for variables.
#[derive(Clone, Debug, Default)]
pub struct ExprBuilder {
    declarations: HashSet<String>,
}

impl ExprBuilder {
    pub fn new() -> Self {
        Self {
            declarations: Default::default(),
        }
    }

    pub fn bool_const(&self, value: bool) -> Box<BoolExpr> {
        Box::new(BoolExpr::BoolLit(value))
    }

    pub fn int_const(&self, value: i64) -> Box<NumExpr> {
        Box::new(NumExpr::IntLit(value))
    }

    pub fn uint_const(&self, value: u64) -> Box<NumExpr> {
        Box::new(NumExpr::UIntLit(value))
    }

    pub fn float_const(&self, value: f64) -> Box<NumExpr> {
        Box::new(NumExpr::FloatLit(value))
    }

    pub fn bool_var(&mut self, name: String) -> ArgusResult<Box<BoolExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(BoolExpr::BoolVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    pub fn int_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::IntVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    pub fn uint_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::UIntVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    pub fn float_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::FloatVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    pub fn make_neg(&self, arg: Box<NumExpr>) -> Box<NumExpr> {
        Box::new(NumExpr::Neg { arg })
    }

    pub fn make_add<I>(&self, args: I) -> ArgusResult<Box<NumExpr>>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new(NumExpr::Add { args }))
        }
    }

    pub fn make_mul<I>(&self, args: I) -> ArgusResult<Box<NumExpr>>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new(NumExpr::Mul { args }))
        }
    }

    pub fn make_div(&self, dividend: Box<NumExpr>, divisor: Box<NumExpr>) -> Box<NumExpr> {
        Box::new(NumExpr::Div { dividend, divisor })
    }

    pub fn make_cmp(&self, op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Cmp { op, lhs, rhs })
    }

    pub fn make_lt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Less { strict: true }, lhs, rhs)
    }

    pub fn make_le(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Less { strict: false }, lhs, rhs)
    }

    pub fn make_gt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Greater { strict: true }, lhs, rhs)
    }

    pub fn make_ge(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Greater { strict: false }, lhs, rhs)
    }

    pub fn make_eq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Eq, lhs, rhs)
    }

    pub fn make_neq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::NotEq, lhs, rhs)
    }

    pub fn make_not(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Not { arg })
    }

    pub fn make_or<I>(&self, args: I) -> ArgusResult<Box<BoolExpr>>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new(BoolExpr::Or { args }))
        }
    }

    pub fn make_and<I>(&self, args: I) -> ArgusResult<Box<BoolExpr>>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new(BoolExpr::And { args }))
        }
    }
}

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

#[cfg(test)]
pub mod arbitrary {
    //! Helper functions to generate arbitrary expressions using [`proptest`].
    use proptest::prelude::*;

    use super::*;

    pub fn num_expr() -> impl Strategy<Value = Box<NumExpr>> {
        let leaf = prop_oneof![
            any::<i64>().prop_map(|val| Box::new(NumExpr::IntLit(val))),
            any::<u64>().prop_map(|val| Box::new(NumExpr::UIntLit(val))),
            any::<f64>().prop_map(|val| Box::new(NumExpr::FloatLit(val))),
            "[[:word:]]*".prop_map(|name| Box::new(NumExpr::IntVar { name })),
            "[[:word:]]*".prop_map(|name| Box::new(NumExpr::UIntVar { name })),
            "[[:word:]]*".prop_map(|name| Box::new(NumExpr::FloatVar { name })),
        ];

        leaf.prop_recursive(
            8,   // 8 levels deep
            128, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|arg| Box::new(NumExpr::Neg { arg })),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(NumExpr::Add {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(NumExpr::Mul {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                    (inner.clone(), inner)
                        .prop_map(|(dividend, divisor)| { Box::new(NumExpr::Div { dividend, divisor }) })
                ]
            },
        )
    }

    pub fn cmp_expr() -> impl Strategy<Value = Box<BoolExpr>> {
        use Ordering::*;
        let op = prop_oneof![Just(Eq), Just(NotEq),];
        let lhs = num_expr();
        let rhs = num_expr();

        (op, lhs, rhs).prop_map(|(op, lhs, rhs)| Box::new(BoolExpr::Cmp { op, lhs, rhs }))
    }

    pub fn bool_expr() -> impl Strategy<Value = Box<BoolExpr>> {
        let leaf = prop_oneof![
            any::<bool>().prop_map(|val| Box::new(BoolExpr::BoolLit(val))),
            "[[:word:]]*".prop_map(|name| Box::new(BoolExpr::BoolVar { name })),
            cmp_expr(),
        ];

        leaf.prop_recursive(
            8,   // 8 levels deep
            128, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|arg| Box::new(BoolExpr::Not { arg })),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(BoolExpr::And {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                    prop::collection::vec(inner, 0..10).prop_map(|args| {
                        Box::new(BoolExpr::Or {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                ]
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use paste::paste;
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn correctly_create_num_expr(num_expr in arbitrary::num_expr()) {
            _ = num_expr;
        }
    }

    proptest! {
        #[test]
        fn correctly_create_bool_expr(bool_expr in arbitrary::bool_expr()) {
            _ = bool_expr;
        }
    }

    proptest! {
        #[test]
        fn neg_num_expr(arg in arbitrary::num_expr()) {
            let expr = -arg;
            assert!(matches!(expr, NumExpr::Neg { arg: _ }));
        }
    }

    macro_rules! test_num_binop {
        ($name:ident, $method:ident with /) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _num_expr>](lhs in arbitrary::num_expr(), rhs in arbitrary::num_expr()) {
                        let expr = lhs / rhs;
                        assert!(matches!(expr, NumExpr::$name {dividend: _, divisor: _ }));
                    }
                }
            }
        };
        ($name:ident, $method:ident with $op:tt) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _num_expr>](lhs in arbitrary::num_expr(), rhs in arbitrary::num_expr()) {
                        let expr = lhs $op rhs;
                        assert!(matches!(expr, NumExpr::$name { args: _ }));
                    }
                }
            }
        };
    }

    test_num_binop!(Add, add with +);
    test_num_binop!(Mul, mul with *);
    test_num_binop!(Div, div with /);

    proptest! {
        #[test]
        fn not_bool_expr(arg in arbitrary::bool_expr()) {
            let expr = !arg;
            assert!(matches!(expr, BoolExpr::Not { arg: _ }));
        }
    }

    macro_rules! test_bool_binop {
        ($name:ident, $method:ident with $op:tt) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _bool_expr>](lhs in arbitrary::bool_expr(), rhs in arbitrary::bool_expr()) {
                        let expr = Box::new(lhs $op rhs);
                        assert!(matches!(*expr, BoolExpr::$name { args: _ }));
                    }
                }
            }
        };
    }

    test_bool_binop!(And, bitand with &);
    test_bool_binop!(Or, bitor with |);
}
