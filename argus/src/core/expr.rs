//! Expression tree for Argus specifications

use hashbrown::HashMap;

mod bool_expr;
pub mod iter;
mod num_expr;
mod traits;

pub use bool_expr::*;
use enum_dispatch::enum_dispatch;
pub use num_expr::*;
pub use traits::*;

use self::iter::AstIter;
use crate::{ArgusResult, Error, Type};

/// A trait representing expressions
#[enum_dispatch]
pub trait AnyExpr {
    /// Check if the given expression is a numeric expression
    fn is_numeric(&self) -> bool;
    /// Check if the given expression is a boolean expression
    fn is_boolean(&self) -> bool;
    /// Get the arguments to the current expression.
    ///
    /// If the expression doesn't contain arguments (i.e., it is a leaf expression) then
    /// the vector is empty.
    fn args(&self) -> Vec<ExprRef<'_>>;
}

/// Marker trait for numeric expressions
pub trait IsNumExpr: AnyExpr + Into<NumExpr> {}

/// Marker trait for Boolean expressions
pub trait IsBoolExpr: AnyExpr + Into<BoolExpr> {}

/// All expressions that are numeric
#[derive(
    Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display, derive_more::From, derive_more::TryInto,
)]
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

impl AnyExpr for NumExpr {
    fn is_numeric(&self) -> bool {
        true
    }
    fn is_boolean(&self) -> bool {
        false
    }
    fn args(&self) -> Vec<ExprRef<'_>> {
        match self {
            NumExpr::IntLit(expr) => expr.args(),
            NumExpr::UIntLit(expr) => expr.args(),
            NumExpr::FloatLit(expr) => expr.args(),
            NumExpr::IntVar(expr) => expr.args(),
            NumExpr::UIntVar(expr) => expr.args(),
            NumExpr::FloatVar(expr) => expr.args(),
            NumExpr::Neg(expr) => expr.args(),
            NumExpr::Add(expr) => expr.args(),
            NumExpr::Sub(expr) => expr.args(),
            NumExpr::Mul(expr) => expr.args(),
            NumExpr::Div(expr) => expr.args(),
            NumExpr::Abs(expr) => expr.args(),
        }
    }
}

/// All expressions that are evaluated to be of type `bool`
#[derive(
    Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display, derive_more::From, derive_more::TryInto,
)]
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

impl AnyExpr for BoolExpr {
    fn is_boolean(&self) -> bool {
        true
    }
    fn is_numeric(&self) -> bool {
        false
    }
    fn args(&self) -> Vec<ExprRef<'_>> {
        match self {
            BoolExpr::BoolLit(expr) => expr.args(),
            BoolExpr::BoolVar(expr) => expr.args(),
            BoolExpr::Cmp(expr) => expr.args(),
            BoolExpr::Not(expr) => expr.args(),
            BoolExpr::And(expr) => expr.args(),
            BoolExpr::Or(expr) => expr.args(),
            BoolExpr::Next(expr) => expr.args(),
            BoolExpr::Oracle(expr) => expr.args(),
            BoolExpr::Always(expr) => expr.args(),
            BoolExpr::Eventually(expr) => expr.args(),
            BoolExpr::Until(expr) => expr.args(),
        }
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

/// An expression (either [`BoolExpr`] or [`NumExpr`])
#[derive(Clone, Debug, derive_more::Display, derive_more::From, derive_more::TryInto)]
pub enum Expr {
    /// A reference to a [`BoolExpr`]
    Bool(BoolExpr),
    /// A reference to a [`NumExpr`]
    Num(NumExpr),
}

impl AnyExpr for Expr {
    fn is_numeric(&self) -> bool {
        matches!(self, Expr::Num(_))
    }

    fn is_boolean(&self) -> bool {
        matches!(self, Expr::Bool(_))
    }

    fn args(&self) -> Vec<ExprRef<'_>> {
        match self {
            Expr::Bool(expr) => expr.args(),
            Expr::Num(expr) => expr.args(),
        }
    }
}

/// Expression builder
///
/// The `ExprBuilder` is a factory structure that deals with the creation of
/// expressions. The main goal of this is to ensure users do not create duplicate
/// definitions for variables.
#[derive(Clone, Debug, Default)]
pub struct ExprBuilder {
    pub(crate) declarations: HashMap<String, Type>,
}

impl ExprBuilder {
    /// Create a new `ExprBuilder` context.
    pub fn new() -> Self {
        Self {
            declarations: Default::default(),
        }
    }

    /// Declare a constant boolean expression
    pub fn bool_const(&self, value: bool) -> BoolExpr {
        BoolLit(value).into()
    }

    /// Declare a constant integer expression
    pub fn int_const(&self, value: i64) -> NumExpr {
        IntLit(value).into()
    }

    /// Declare a constant unsigned integer expression
    pub fn uint_const(&self, value: u64) -> NumExpr {
        UIntLit(value).into()
    }

    /// Declare a constant floating point expression
    pub fn float_const(&self, value: f64) -> NumExpr {
        FloatLit(value).into()
    }

    /// Declare a boolean variable
    pub fn bool_var(&mut self, name: String) -> ArgusResult<BoolExpr> {
        match self.declarations.insert(name.clone(), Type::Bool) {
            None | Some(Type::Bool) => Ok((BoolVar { name }).into()),
            _ => Err(Error::IdentifierRedeclaration),
        }
    }

    /// Declare a integer variable
    pub fn int_var(&mut self, name: String) -> ArgusResult<NumExpr> {
        match self.declarations.insert(name.clone(), Type::Int) {
            None | Some(Type::Int) => Ok((IntVar { name }).into()),
            _ => Err(Error::IdentifierRedeclaration),
        }
    }

    /// Declare a unsigned integer variable
    pub fn uint_var(&mut self, name: String) -> ArgusResult<NumExpr> {
        match self.declarations.insert(name.clone(), Type::UInt) {
            None | Some(Type::UInt) => Ok((UIntVar { name }).into()),
            _ => Err(Error::IdentifierRedeclaration),
        }
    }

    /// Declare a floating point variable
    pub fn float_var(&mut self, name: String) -> ArgusResult<NumExpr> {
        match self.declarations.insert(name.clone(), Type::Float) {
            None | Some(Type::Float) => Ok((FloatVar { name }).into()),
            _ => Err(Error::IdentifierRedeclaration),
        }
    }

    /// Create a [`NumExpr::Neg`] expression
    pub fn make_neg(&self, arg: Box<NumExpr>) -> NumExpr {
        (Neg { arg }).into()
    }

    /// Create a [`NumExpr::Add`] expression
    pub fn make_add<I>(&self, args: I) -> ArgusResult<NumExpr>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let mut new_args = Vec::<NumExpr>::new();
        for arg in args.into_iter() {
            // Flatten the args if there is an Add
            if let NumExpr::Add(Add { args }) = arg {
                new_args.extend(args.into_iter());
            } else {
                new_args.push(arg);
            }
        }
        if new_args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok((Add { args: new_args }).into())
        }
    }

    /// Create a [`NumExpr::Mul`] expression
    pub fn make_mul<I>(&self, args: I) -> ArgusResult<NumExpr>
    where
        I: IntoIterator<Item = NumExpr>,
    {
        let mut new_args = Vec::<NumExpr>::new();
        for arg in args.into_iter() {
            // Flatten the args if there is a Mul
            if let NumExpr::Mul(Mul { args }) = arg {
                new_args.extend(args.into_iter());
            } else {
                new_args.push(arg);
            }
        }
        if new_args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok((Mul { args: new_args }).into())
        }
    }

    /// Create a [`NumExpr::Sub`] expression
    pub fn make_sub(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> NumExpr {
        (Sub { lhs, rhs }).into()
    }

    /// Create a [`NumExpr::Div`] expression
    pub fn make_div(&self, dividend: Box<NumExpr>, divisor: Box<NumExpr>) -> NumExpr {
        (Div { dividend, divisor }).into()
    }

    /// Create a [`BoolExpr::Cmp`] expression
    pub fn make_cmp(&self, op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        (Cmp { op, lhs, rhs }).into()
    }

    /// Create a "less than" ([`BoolExpr::Cmp`]) expression
    pub fn make_lt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::Less { strict: true }, lhs, rhs)
    }

    /// Create a "less than or equal" ([`BoolExpr::Cmp`]) expression
    pub fn make_le(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::Less { strict: false }, lhs, rhs)
    }

    /// Create a "greater than" ([`BoolExpr::Cmp`]) expression
    pub fn make_gt(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::Greater { strict: true }, lhs, rhs)
    }

    /// Create a "greater than or equal" ([`BoolExpr::Cmp`]) expression
    pub fn make_ge(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::Greater { strict: false }, lhs, rhs)
    }

    /// Create a "equals" ([`BoolExpr::Cmp`]) expression
    pub fn make_eq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::Eq, lhs, rhs)
    }

    /// Create a "not equals" ([`BoolExpr::Cmp`]) expression
    pub fn make_neq(&self, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> BoolExpr {
        self.make_cmp(Ordering::NotEq, lhs, rhs)
    }

    /// Create a [`BoolExpr::Not`] expression.
    pub fn make_not(&self, arg: Box<BoolExpr>) -> BoolExpr {
        (Not { arg }).into()
    }

    /// Create a [`BoolExpr::Or`] expression.
    pub fn make_or<I>(&self, args: I) -> ArgusResult<BoolExpr>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let mut new_args = Vec::<BoolExpr>::new();
        for arg in args.into_iter() {
            // Flatten the args if there is an Or
            if let BoolExpr::Or(Or { args }) = arg {
                new_args.extend(args.into_iter());
            } else {
                new_args.push(arg);
            }
        }
        if new_args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok((Or { args: new_args }).into())
        }
    }

    /// Create a [`BoolExpr::And`] expression.
    pub fn make_and<I>(&self, args: I) -> ArgusResult<BoolExpr>
    where
        I: IntoIterator<Item = BoolExpr>,
    {
        let mut new_args = Vec::<BoolExpr>::new();
        for arg in args.into_iter() {
            // Flatten the args if there is an And
            if let BoolExpr::And(And { args }) = arg {
                new_args.extend(args.into_iter());
            } else {
                new_args.push(arg);
            }
        }
        if new_args.len() < 2 {
            Err(Error::IncompleteArgs)
        } else {
            Ok((And { args: new_args }).into())
        }
    }

    /// Create an expression equivalent to `lhs -> rhs`.
    ///
    /// This essentially breaks down the expression as `~lhs | rhs`.
    #[allow(clippy::boxed_local)]
    pub fn make_implies(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> ArgusResult<BoolExpr> {
        let np = self.make_not(lhs);
        self.make_or([np, *rhs])
    }

    /// Create an expression equivalent to `lhs <-> rhs`.
    ///
    /// This essentially breaks down the expression as `(lhs & rhs) | (~lhs & ~rhs)`.
    pub fn make_equiv(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> ArgusResult<BoolExpr> {
        let np = self.make_not(lhs.clone());
        let nq = self.make_not(rhs.clone());

        let npnq = self.make_and([np, nq])?;
        let pq = self.make_and([*lhs, *rhs])?;

        self.make_or([pq, npnq])
    }

    /// Create an expression equivalent to `lhs ^ rhs`.
    ///
    /// This essentially breaks down the expression as `~(lhs <-> rhs)`.
    pub fn make_xor(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> ArgusResult<BoolExpr> {
        Ok(self.make_not(Box::new(self.make_equiv(lhs, rhs)?)))
    }

    /// Create a [`BoolExpr::Next`] expression.
    pub fn make_next(&self, arg: Box<BoolExpr>) -> BoolExpr {
        (Next { arg }).into()
    }

    /// Create a [`BoolExpr::Oracle`] expression.
    pub fn make_oracle(&self, steps: usize, arg: Box<BoolExpr>) -> BoolExpr {
        if steps == 1 {
            self.make_next(arg)
        } else {
            (Oracle { steps, arg }).into()
        }
    }

    /// Create a [`BoolExpr::Always`] expression.
    pub fn make_always(&self, arg: Box<BoolExpr>) -> BoolExpr {
        (Always {
            arg,
            interval: (..).into(),
        })
        .into()
    }

    /// Create a [`BoolExpr::Always`] expression with an interval.
    pub fn make_timed_always(&self, interval: Interval, arg: Box<BoolExpr>) -> BoolExpr {
        (Always { arg, interval }).into()
    }

    /// Create a [`BoolExpr::Eventually`] expression.
    pub fn make_eventually(&self, arg: Box<BoolExpr>) -> BoolExpr {
        (Eventually {
            arg,
            interval: (..).into(),
        })
        .into()
    }

    /// Create a [`BoolExpr::Eventually`] expression with an interval.
    pub fn make_timed_eventually(&self, interval: Interval, arg: Box<BoolExpr>) -> BoolExpr {
        (Eventually { arg, interval }).into()
    }

    /// Create a [`BoolExpr::Until`] expression.
    pub fn make_until(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> BoolExpr {
        (Until {
            lhs,
            rhs,
            interval: (..).into(),
        })
        .into()
    }

    /// Create a [`BoolExpr::Until`] expression with an interval.
    pub fn make_timed_until(&self, interval: Interval, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> BoolExpr {
        BoolExpr::from(Until { lhs, rhs, interval })
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

        #[allow(clippy::arc_with_non_send_sync)]
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
        #[allow(clippy::arc_with_non_send_sync)]
        let leaf = prop_oneof![
            any::<bool>().prop_map(|val| Box::new(BoolLit(val).into())),
            "[[:word:]]*".prop_map(|name| Box::new((BoolVar { name }).into())),
            cmp_expr(),
        ];

        #[allow(clippy::arc_with_non_send_sync)]
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

    #[test]
    fn is_numeric() {
        let mut builder = ExprBuilder::new();
        let a = builder.int_const(10);
        let b = builder.int_var("b".to_owned()).unwrap();
        let spec = a + b;

        assert!(spec.is_numeric());
        assert!(!spec.is_boolean());
    }

    #[test]
    fn is_boolean() {
        let mut builder = ExprBuilder::new();
        let a = builder.bool_const(true);
        let b = builder.bool_var("b".to_owned()).unwrap();
        let spec = a & b;

        assert!(!spec.is_numeric());
        assert!(spec.is_boolean());
    }
}
