//! Expression tree for Argus specifications

use std::collections::HashSet;

mod bool_expr;
pub mod iter;
mod num_expr;
mod traits;

pub use bool_expr::*;
use enum_dispatch::enum_dispatch;
pub use num_expr::*;
pub use traits::*;

use self::iter::AstIter;
use crate::{ArgusResult, Error};

/// All expressions that are numeric
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr)]
#[enum_dispatch(Expr)]
pub enum NumExpr {
    /// A signed integer literal
    IntLit(IntLit),
    /// An unsigned integer literal
    UIntLit(UIntLit),
    /// A floating point literal
    FloatLit(FloatLit),
    /// A signed integer variable
    IntVar(IntVar),
    /// A unsigned integer variable
    UIntVar(UIntVar),
    /// A floating point number variable
    FloatVar(FloatVar),
    /// Numeric negation of a numeric expression
    Neg(Neg),
    /// Arithmetic addition of a list of numeric expressions
    Add(Add),
    /// Subtraction of two numbers
    Sub(Sub),
    /// Arithmetic multiplication of a list of numeric expressions
    Mul(Mul),
    /// Divide two expressions `dividend / divisor`
    Div(Div),
    /// The absolute value of an expression
    Abs(Abs),
}

impl NumExpr {
    /// Create a borrowed iterator over the expression tree
    pub fn iter(&self) -> AstIter<'_> {
        AstIter::new(self.into())
    }
}

/// All expressions that are evaluated to be of type `bool`
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr)]
#[enum_dispatch(Expr)]
pub enum BoolExpr {
    /// A `bool` literal
    BoolLit(BoolLit),
    /// A `bool` variable
    BoolVar(BoolVar),
    /// A comparison expression
    Cmp(Cmp),
    /// Logical negation of an expression
    Not(Not),
    /// Logical conjunction of a list of expressions
    And(And),
    /// Logical disjunction of a list of expressions
    Or(Or),

    /// A temporal next expression
    ///
    /// Checks if the next time point in a signal is `true` or not.
    Next(Next),

    /// Temporal "oracle" expression
    ///
    /// This is equivalent to `steps` number of nested [`Next`](BoolExpr::Next)
    /// expressions.
    Oracle(Oracle),

    /// A temporal always expression
    ///
    /// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0,
    ///   Unbounded)`: checks if the signal is `true` for all points in a signal.
    /// - Otherwise: checks if the signal is `true` for all points within the
    ///   `interval`.
    Always(Always),

    /// A temporal eventually expression
    ///
    /// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0,
    ///   Unbounded)`: checks if the signal is `true` for some point in a signal.
    /// - Otherwise: checks if the signal is `true` for some point within the
    ///   `interval`.
    Eventually(Eventually),

    /// A temporal until expression
    ///
    /// Checks if the `lhs` is always `true` for a signal until `rhs` becomes `true`.
    Until(Until),
}

impl BoolExpr {
    /// Create a borrowed iterator over the expression tree
    pub fn iter(&self) -> AstIter<'_> {
        AstIter::new(self.into())
    }
}

/// A reference to an expression (either [`BoolExpr`] or [`NumExpr`]).
#[derive(Clone, Copy, Debug, derive_more::From)]
pub enum ExprRef<'a> {
    /// A reference to a [`BoolExpr`]
    Bool(&'a BoolExpr),
    /// A reference to a [`NumExpr`]
    Num(&'a NumExpr),
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
    /// Create a new `ExprBuilder` context.
    pub fn new() -> Self {
        Self {
            declarations: Default::default(),
        }
    }

    /// Declare a constant boolean expression
    pub fn bool_const(&self, value: bool) -> Box<BoolExpr> {
        Box::new(BoolLit(value).into())
    }

    /// Declare a constant integer expression
    pub fn int_const(&self, value: i64) -> Box<NumExpr> {
        Box::new(IntLit(value).into())
    }

    /// Declare a constant unsigned integer expression
    pub fn uint_const(&self, value: u64) -> Box<NumExpr> {
        Box::new(UIntLit(value).into())
    }

    /// Declare a constant floating point expression
    pub fn float_const(&self, value: f64) -> Box<NumExpr> {
        Box::new(FloatLit(value).into())
    }

    /// Declare a boolean variable
    pub fn bool_var(&mut self, name: String) -> ArgusResult<Box<BoolExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new((BoolVar { name }).into()))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a integer variable
    pub fn int_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new((IntVar { name }).into()))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a unsigned integer variable
    pub fn uint_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new((UIntVar { name }).into()))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a floating point variable
    pub fn float_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new((FloatVar { name }).into()))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Create a [`NumExpr::Neg`] expression
    pub fn make_neg(&self, arg: Box<NumExpr>) -> Box<NumExpr> {
        Box::new((Neg { arg }).into())
    }

    /// Create a [`NumExpr::Add`] expression
    pub fn make_add<I>(&self, args: I) -> ArgusResult<Box<NumExpr>>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new((Add { args }).into()))
        }
    }

    /// Create a [`NumExpr::Mul`] expression
    pub fn make_mul<I>(&self, args: I) -> ArgusResult<Box<NumExpr>>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new((Mul { args }).into()))
        }
    }

    /// Create a [`NumExpr::Div`] expression
    pub fn make_div(&self, dividend: Box<NumExpr>, divisor: Box<NumExpr>) -> Box<NumExpr> {
        Box::new((Div { dividend, divisor }).into())
    }

    /// Create a [`BoolExpr::Cmp`] expression
    pub fn make_cmp(&self, op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        Box::new((Cmp { op, lhs, rhs }).into())
    }

    /// Create a "less than" ([`BoolExpr::Cmp`]) expression
    pub fn make_lt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Less { strict: true }, lhs, rhs)
    }

    /// Create a "less than or equal" ([`BoolExpr::Cmp`]) expression
    pub fn make_le(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Less { strict: false }, lhs, rhs)
    }

    /// Create a "greater than" ([`BoolExpr::Cmp`]) expression
    pub fn make_gt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Greater { strict: true }, lhs, rhs)
    }

    /// Create a "greater than or equal" ([`BoolExpr::Cmp`]) expression
    pub fn make_ge(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Greater { strict: false }, lhs, rhs)
    }

    /// Create a "equals" ([`BoolExpr::Cmp`]) expression
    pub fn make_eq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::Eq, lhs, rhs)
    }

    /// Create a "not equals" ([`BoolExpr::Cmp`]) expression
    pub fn make_neq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        self.make_cmp(Ordering::NotEq, lhs, rhs)
    }

    /// Create a [`BoolExpr::Not`] expression.
    pub fn make_not(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Not { arg }).into())
    }

    /// Create a [`BoolExpr::Or`] expression.
    pub fn make_or<I>(&self, args: I) -> ArgusResult<Box<BoolExpr>>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new((Or { args }).into()))
        }
    }

    /// Create a [`BoolExpr::And`] expression.
    pub fn make_and<I>(&self, args: I) -> ArgusResult<Box<BoolExpr>>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok(Box::new((And { args }).into()))
        }
    }

    /// Create a [`BoolExpr::Next`] expression.
    pub fn make_next(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Next { arg }).into())
    }

    /// Create a [`BoolExpr::Oracle`] expression.
    pub fn make_oracle(&self, steps: usize, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Oracle { steps, arg }).into())
    }

    /// Create a [`BoolExpr::Always`] expression.
    pub fn make_always(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(
            (Always {
                arg,
                interval: (..).into(),
            })
            .into(),
        )
    }

    /// Create a [`BoolExpr::Always`] expression with an interval.
    pub fn make_timed_always(&self, interval: Interval, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Always { arg, interval }).into())
    }

    /// Create a [`BoolExpr::Eventually`] expression.
    pub fn make_eventually(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(
            (Eventually {
                arg,
                interval: (..).into(),
            })
            .into(),
        )
    }

    /// Create a [`BoolExpr::Eventually`] expression with an interval.
    pub fn make_timed_eventually(&self, interval: Interval, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Eventually { arg, interval }).into())
    }

    /// Create a [`BoolExpr::Until`] expression.
    pub fn make_until(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(
            (Until {
                lhs,
                rhs,
                interval: (..).into(),
            })
            .into(),
        )
    }

    /// Create a [`BoolExpr::Until`] expression with an interval.
    pub fn make_timed_until(&self, interval: Interval, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new((Until { lhs, rhs, interval }).into())
    }
}

#[cfg(any(test, feature = "arbitrary"))]
pub mod arbitrary {

    //! Helper functions to generate arbitrary expressions using [`mod@proptest`].
    use core::ops::Bound;
    use core::time::Duration;

    use proptest::prelude::*;

    use super::*;

    /// Generate arbitrary numeric expressions
    pub fn num_expr() -> impl Strategy<Value = Box<NumExpr>> {
        let leaf = prop_oneof![
            any::<i64>().prop_map(|val| Box::new(IntLit(val).into())),
            any::<u64>().prop_map(|val| Box::new(UIntLit(val).into())),
            any::<f64>().prop_map(|val| Box::new(FloatLit(val).into())),
            "[[:word:]]*".prop_map(|name| Box::new((IntVar { name }).into())),
            "[[:word:]]*".prop_map(|name| Box::new((UIntVar { name }).into())),
            "[[:word:]]*".prop_map(|name| Box::new((FloatVar { name }).into())),
        ];

        leaf.prop_recursive(
            8,   // 8 levels deep
            128, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|arg| Box::new((Neg { arg }).into())),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(
                            (Add {
                                args: args.into_iter().map(|arg| *arg).collect(),
                            })
                            .into(),
                        )
                    }),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(
                            (Mul {
                                args: args.into_iter().map(|arg| *arg).collect(),
                            })
                            .into(),
                        )
                    }),
                    (inner.clone(), inner)
                        .prop_map(|(dividend, divisor)| { Box::new((Div { dividend, divisor }).into()) })
                ]
            },
        )
    }

    /// Generate arbitrary comparison expressions
    pub fn cmp_expr() -> impl Strategy<Value = Box<BoolExpr>> {
        use Ordering::*;
        let op = prop_oneof![Just(Eq), Just(NotEq),];
        let lhs = num_expr();
        let rhs = num_expr();

        (op, lhs, rhs).prop_map(|(op, lhs, rhs)| Box::new((Cmp { op, lhs, rhs }).into()))
    }

    /// Generate arbitrary boolean expressions
    pub fn bool_expr() -> impl Strategy<Value = Box<BoolExpr>> {
        let leaf = prop_oneof![
            any::<bool>().prop_map(|val| Box::new(BoolLit(val).into())),
            "[[:word:]]*".prop_map(|name| Box::new((BoolVar { name }).into())),
            cmp_expr(),
        ];

        leaf.prop_recursive(
            8,   // 8 levels deep
            128, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                let interval = (any::<(Bound<Duration>, Bound<Duration>)>()).prop_map_into::<Interval>();
                prop_oneof![
                    inner.clone().prop_map(|arg| Box::new((Not { arg }).into())),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(
                            (And {
                                args: args.into_iter().map(|arg| *arg).collect(),
                            })
                            .into(),
                        )
                    }),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(
                            (Or {
                                args: args.into_iter().map(|arg| *arg).collect(),
                            })
                            .into(),
                        )
                    }),
                    inner.clone().prop_map(|arg| Box::new((Next { arg }).into())),
                    (inner.clone(), interval.clone())
                        .prop_map(|(arg, interval)| Box::new((Always { arg, interval }).into())),
                    (inner.clone(), interval.clone())
                        .prop_map(|(arg, interval)| Box::new((Eventually { arg, interval }).into())),
                    (inner.clone(), inner, interval)
                        .prop_map(|(lhs, rhs, interval)| Box::new((Until { lhs, rhs, interval }).into())),
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
            let expr = -*arg;
            assert!(matches!(expr, NumExpr::Neg(Neg { arg: _ })));
        }
    }

    macro_rules! test_num_binop {
        ($name:ident, $method:ident with /) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _num_expr>](lhs in arbitrary::num_expr(), rhs in arbitrary::num_expr()) {
                        let expr = *lhs / *rhs;
                        assert!(matches!(expr, NumExpr::$name($name {dividend: _, divisor: _ })));
                    }
                }
            }
        };
        ($name:ident, $method:ident with $op:tt) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _num_expr>](lhs in arbitrary::num_expr(), rhs in arbitrary::num_expr()) {
                        let expr = *lhs $op *rhs;
                        assert!(matches!(expr, NumExpr::$name($name { args: _ })));
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
            let expr = !*arg;
            assert!(matches!(expr, BoolExpr::Not(Not { arg: _ })));
        }
    }

    macro_rules! test_bool_binop {
        ($name:ident, $method:ident with $op:tt) => {
            paste! {
                proptest! {
                    #[test]
                    fn [<$method _bool_expr>](lhs in arbitrary::bool_expr(), rhs in arbitrary::bool_expr()) {
                        let expr = *lhs $op *rhs;
                        assert!(matches!(expr, BoolExpr::$name($name { args: _ })));
                    }
                }
            }
        };
    }

    test_bool_binop!(And, bitand with &);
    test_bool_binop!(Or, bitor with |);
}
