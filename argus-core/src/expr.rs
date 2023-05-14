//! Expression tree for Argus specifications

use std::any::Any;
use std::collections::HashSet;
use std::ops::{Bound, RangeBounds};
use std::time::Duration;

mod bool_ops;
mod internal_macros;
pub mod iter;
mod num_ops;
mod traits;

pub use bool_ops::*;
pub use num_ops::*;
pub use traits::*;

use self::iter::AstIter;
use crate::{ArgusResult, Error};

/// All expressions that are numeric
#[derive(Clone, Debug, PartialEq)]
pub enum NumExpr {
    /// A signed integer literal
    IntLit(i64),
    /// An unsigned integer literal
    UIntLit(u64),
    /// A floating point literal
    FloatLit(f64),
    /// A signed integer variable
    IntVar {
        /// Name of the variable
        name: String,
    },
    /// A unsigned integer variable
    UIntVar {
        /// Name of the variable
        name: String,
    },
    /// A floating point number variable
    FloatVar {
        /// Name of the variable
        name: String,
    },
    /// Numeric negation of a numeric expression
    Neg {
        /// Numeric expression being negated
        arg: Box<NumExpr>,
    },
    /// Arithmetic addition of a list of numeric expressions
    Add {
        /// List of expressions being added
        args: Vec<NumExpr>,
    },
    /// Subtraction of two numbers
    Sub {
        /// LHS to the expression `lhs - rhs`
        lhs: Box<NumExpr>,
        /// RHS to the expression `lhs - rhs`
        rhs: Box<NumExpr>,
    },
    /// Arithmetic multiplication of a list of numeric expressions
    Mul {
        /// List of expressions being multiplied
        args: Vec<NumExpr>,
    },
    /// Divide two expressions `dividend / divisor`
    Div {
        /// The dividend
        dividend: Box<NumExpr>,
        /// The divisor
        divisor: Box<NumExpr>,
    },
    /// The absolute value of an expression
    Abs {
        /// Argument to `abs`
        arg: Box<NumExpr>,
    },
}

impl Expr for NumExpr {
    fn is_numeric(&self) -> bool {
        true
    }

    fn is_boolean(&self) -> bool {
        false
    }

    fn args(&self) -> Vec<ExprRef<'_>> {
        match self {
            NumExpr::Neg { arg } => vec![arg.as_ref().into()],
            NumExpr::Add { args } | NumExpr::Mul { args } => args.iter().map(|arg| arg.into()).collect(),
            NumExpr::Div { dividend, divisor } => vec![dividend.as_ref().into(), divisor.as_ref().into()],
            _ => vec![],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn iter(&self) -> iter::AstIter<'_> {
        AstIter::new(self.into())
    }
}

/// Types of comparison operations
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ordering {
    /// Equality check for two expressions
    Eq,
    /// Non-equality check for two expressions
    NotEq,
    /// Less than check
    Less {
        /// Denotes `lhs < rhs` if `strict`, and `lhs <= rhs` otherwise.
        strict: bool,
    },
    /// Greater than check
    Greater {
        /// Denotes `lhs > rhs` if `strict`, and `lhs >= rhs` otherwise.
        strict: bool,
    },
}

impl Ordering {
    /// Check if `Ordering::Eq`
    pub fn equal() -> Self {
        Self::Eq
    }

    /// Check if `Ordering::NotEq`
    pub fn not_equal() -> Self {
        Self::NotEq
    }

    /// Check if `Ordering::Less { strict: true }`
    pub fn less_than() -> Self {
        Self::Less { strict: true }
    }

    /// Check if `Ordering::Less { strict: false }`
    pub fn less_than_eq() -> Self {
        Self::Less { strict: false }
    }

    /// Check if `Ordering::Greater { strict: true }`
    pub fn greater_than() -> Self {
        Self::Greater { strict: true }
    }

    /// Check if `Ordering::Less { strict: false }`
    pub fn greater_than_eq() -> Self {
        Self::Greater { strict: false }
    }
}

/// A time interval for a temporal expression.
#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Into)]
#[into(owned, ref, ref_mut)]
pub struct Interval {
    /// Start of the interval
    pub start: Bound<Duration>,
    /// End of the interval
    pub end: Bound<Duration>,
}

impl Interval {
    /// Create a new interval
    ///
    /// # Note
    ///
    /// Argus doesn't permit `Interval`s with [`Bound::Excluded(_)`] values (as these
    /// can't be monitored reliably) and thus converts all such bounds to an
    /// [`Bound::Included(_)`]. Moreover, if the `start` bound is [`Bound::Unbounded`],
    /// it will get transformed to [`Bound::Included(Duration::ZERO)`].
    pub fn new(start: Bound<Duration>, end: Bound<Duration>) -> Self {
        use Bound::*;
        let start = match start {
            a @ Included(_) => a,
            Excluded(b) => Included(b),
            Unbounded => Included(Duration::ZERO),
        };

        let end = match end {
            Excluded(b) => Included(b),
            bound => bound,
        };

        Self { start, end }
    }

    /// Check if the interval is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        use Bound::*;
        match (&self.start, &self.end) {
            (Included(a), Included(b)) => a > b,
            (Included(a), Excluded(b)) | (Excluded(a), Included(b)) | (Excluded(a), Excluded(b)) => a >= b,
            (Unbounded, Excluded(b)) => b == &Duration::ZERO,
            _ => false,
        }
    }

    /// Check if the interval is a singleton
    ///
    /// This implies that only 1 timepoint is valid within this interval.
    #[inline]
    pub fn is_singleton(&self) -> bool {
        use Bound::*;
        match (&self.start, &self.end) {
            (Included(a), Included(b)) => a == b,
            (Unbounded, Included(b)) => b == &Duration::ZERO,
            _ => false,
        }
    }

    /// Check if the interval covers `[0, ..)`.
    #[inline]
    pub fn is_untimed(&self) -> bool {
        use Bound::*;
        match (self.start, self.end) {
            (Unbounded, Unbounded) | (Included(Duration::ZERO), Unbounded) => true,
            (Included(_), Included(_)) | (Included(_), Unbounded) => false,
            (Excluded(_), _) | (_, Excluded(_)) | (Unbounded, _) => {
                unreachable!("looks like someone didn't use Interval::new")
            }
        }
    }
}

impl<T> From<T> for Interval
where
    T: RangeBounds<Duration>,
{
    fn from(value: T) -> Self {
        Self::new(value.start_bound().cloned(), value.end_bound().cloned())
    }
}

/// All expressions that are evaluated to be of type `bool`
#[derive(Clone, Debug, PartialEq)]
pub enum BoolExpr {
    /// A `bool` literal
    BoolLit(bool),
    /// A `bool` variable
    BoolVar {
        /// Variable name
        name: String,
    },
    /// A comparison expression
    Cmp {
        /// The type of comparison
        op: Ordering,
        /// The LHS for the comparison
        lhs: Box<NumExpr>,
        /// The RHS for the comparison
        rhs: Box<NumExpr>,
    },
    /// Logical negation of an expression
    Not {
        /// Expression to be negated
        arg: Box<BoolExpr>,
    },
    /// Logical conjunction of a list of expressions
    And {
        /// Expressions to be "and"-ed
        args: Vec<BoolExpr>,
    },
    /// Logical disjunction of a list of expressions
    Or {
        /// Expressions to be "or"-ed
        args: Vec<BoolExpr>,
    },

    /// A temporal next expression
    ///
    /// Checks if the next time point in a signal is `true` or not.
    Next {
        /// Argument for `Next`
        arg: Box<BoolExpr>,
    },

    /// Temporal "oracle" expression
    ///
    /// This is equivalent to `steps` number of nested [`Next`](BoolExpr::Next)
    /// expressions.
    Oracle {
        /// Number of steps to look ahead
        steps: usize,
        /// Argument for `Oracle`
        arg: Box<BoolExpr>,
    },

    /// A temporal always expression
    ///
    /// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0,
    ///   Unbounded)`: checks if the signal is `true` for all points in a signal.
    /// - Otherwise: checks if the signal is `true` for all points within the
    ///   `interval`.
    Always {
        /// Argument for `Always`
        arg: Box<BoolExpr>,
        /// Interval for the expression
        interval: Interval,
    },

    /// A temporal eventually expression
    ///
    /// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0,
    ///   Unbounded)`: checks if the signal is `true` for some point in a signal.
    /// - Otherwise: checks if the signal is `true` for some point within the
    ///   `interval`.
    Eventually {
        /// Argument for `Eventually`
        arg: Box<BoolExpr>,
        /// Interval for the expression
        interval: Interval,
    },

    /// A temporal until expression
    ///
    /// Checks if the `lhs` is always `true` for a signal until `rhs` becomes `true`.
    Until {
        /// LHS to `lhs Until rhs`
        lhs: Box<BoolExpr>,
        /// RHS to `lhs Until rhs`
        rhs: Box<BoolExpr>,
        /// Interval for the expression
        interval: Interval,
    },
}

impl Expr for BoolExpr {
    fn is_numeric(&self) -> bool {
        false
    }

    fn is_boolean(&self) -> bool {
        true
    }

    fn args(&self) -> Vec<ExprRef<'_>> {
        match self {
            BoolExpr::Cmp { op: _, lhs, rhs } => vec![lhs.as_ref().into(), rhs.as_ref().into()],
            BoolExpr::Not { arg } => vec![arg.as_ref().into()],
            BoolExpr::And { args } | BoolExpr::Or { args } => args.iter().map(|arg| arg.into()).collect(),
            _ => vec![],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn iter(&self) -> AstIter<'_> {
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
        Box::new(BoolExpr::BoolLit(value))
    }

    /// Declare a constant integer expression
    pub fn int_const(&self, value: i64) -> Box<NumExpr> {
        Box::new(NumExpr::IntLit(value))
    }

    /// Declare a constant unsigned integer expression
    pub fn uint_const(&self, value: u64) -> Box<NumExpr> {
        Box::new(NumExpr::UIntLit(value))
    }

    /// Declare a constant floating point expression
    pub fn float_const(&self, value: f64) -> Box<NumExpr> {
        Box::new(NumExpr::FloatLit(value))
    }

    /// Declare a boolean variable
    pub fn bool_var(&mut self, name: String) -> ArgusResult<Box<BoolExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(BoolExpr::BoolVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a integer variable
    pub fn int_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::IntVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a unsigned integer variable
    pub fn uint_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::UIntVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Declare a floating point variable
    pub fn float_var(&mut self, name: String) -> ArgusResult<Box<NumExpr>> {
        if self.declarations.insert(name.clone()) {
            Ok(Box::new(NumExpr::FloatVar { name }))
        } else {
            Err(Error::IdentifierRedeclaration)
        }
    }

    /// Create a [`NumExpr::Neg`] expression
    pub fn make_neg(&self, arg: Box<NumExpr>) -> Box<NumExpr> {
        Box::new(NumExpr::Neg { arg })
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
            Ok(Box::new(NumExpr::Add { args }))
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
            Ok(Box::new(NumExpr::Mul { args }))
        }
    }

    /// Create a [`NumExpr::Div`] expression
    pub fn make_div(&self, dividend: Box<NumExpr>, divisor: Box<NumExpr>) -> Box<NumExpr> {
        Box::new(NumExpr::Div { dividend, divisor })
    }

    /// Create a [`BoolExpr::Cmp`] expression
    pub fn make_cmp(&self, op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Cmp { op, lhs, rhs })
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
        Box::new(BoolExpr::Not { arg })
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
            Ok(Box::new(BoolExpr::Or { args }))
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
            Ok(Box::new(BoolExpr::And { args }))
        }
    }

    /// Create a [`BoolExpr::Next`] expression.
    pub fn make_next(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Next { arg })
    }

    /// Create a [`BoolExpr::Oracle`] expression.
    pub fn make_oracle(&self, steps: usize, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Oracle { steps, arg })
    }

    /// Create a [`BoolExpr::Always`] expression.
    pub fn make_always(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Always {
            arg,
            interval: (..).into(),
        })
    }

    /// Create a [`BoolExpr::Always`] expression with an interval.
    pub fn make_timed_always(&self, interval: Interval, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Always { arg, interval })
    }

    /// Create a [`BoolExpr::Eventually`] expression.
    pub fn make_eventually(&self, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Eventually {
            arg,
            interval: (..).into(),
        })
    }

    /// Create a [`BoolExpr::Eventually`] expression with an interval.
    pub fn make_timed_eventually(&self, interval: Interval, arg: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Eventually { arg, interval })
    }

    /// Create a [`BoolExpr::Until`] expression.
    pub fn make_until(&self, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Until {
            lhs,
            rhs,
            interval: (..).into(),
        })
    }

    /// Create a [`BoolExpr::Until`] expression with an interval.
    pub fn make_timed_until(&self, interval: Interval, lhs: Box<BoolExpr>, rhs: Box<BoolExpr>) -> Box<BoolExpr> {
        Box::new(BoolExpr::Until { lhs, rhs, interval })
    }
}

#[cfg(any(test, feature = "arbitrary"))]
pub mod arbitrary {
    //! Helper functions to generate arbitrary expressions using [`mod@proptest`].
    use proptest::prelude::*;

    use super::*;

    /// Generate arbitrary numeric expressions
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

    /// Generate arbitrary comparison expressions
    pub fn cmp_expr() -> impl Strategy<Value = Box<BoolExpr>> {
        use Ordering::*;
        let op = prop_oneof![Just(Eq), Just(NotEq),];
        let lhs = num_expr();
        let rhs = num_expr();

        (op, lhs, rhs).prop_map(|(op, lhs, rhs)| Box::new(BoolExpr::Cmp { op, lhs, rhs }))
    }

    /// Generate arbitrary boolean expressions
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
                let interval = (any::<(Bound<Duration>, Bound<Duration>)>()).prop_map_into::<Interval>();
                prop_oneof![
                    inner.clone().prop_map(|arg| Box::new(BoolExpr::Not { arg })),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(BoolExpr::And {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|args| {
                        Box::new(BoolExpr::Or {
                            args: args.into_iter().map(|arg| *arg).collect(),
                        })
                    }),
                    inner.clone().prop_map(|arg| Box::new(BoolExpr::Next { arg })),
                    (inner.clone(), interval.clone())
                        .prop_map(|(arg, interval)| Box::new(BoolExpr::Always { arg, interval })),
                    (inner.clone(), interval.clone())
                        .prop_map(|(arg, interval)| Box::new(BoolExpr::Eventually { arg, interval })),
                    (inner.clone(), inner, interval).prop_map(|(lhs, rhs, interval)| Box::new(BoolExpr::Until {
                        lhs,
                        rhs,
                        interval
                    })),
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
