//! Boolean expression types
use std::ops::{Bound, RangeBounds};
use std::time::Duration;

use itertools::Itertools;

use super::{AnyExpr, BoolExpr, NumExpr};

/// Types of comparison operations
#[derive(Clone, Copy, Debug, PartialEq, derive_more::Display)]
pub enum Ordering {
    /// Equality check for two expressions
    #[display(fmt = "==")]
    Eq,
    /// Non-equality check for two expressions
    #[display(fmt = "!=")]
    NotEq,
    /// Less than check
    #[display(fmt = "{}", r#"if *strict { "<".to_string() } else { "<=".to_string() } "#r)]
    Less {
        /// Denotes `lhs < rhs` if `strict`, and `lhs <= rhs` otherwise.
        strict: bool,
    },
    /// Greater than check
    #[display(fmt = "{}", r#"if *strict { ">".to_string() } else { ">=".to_string() } "#r)]
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

impl core::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start_str = match self.start {
            Bound::Included(b) | Bound::Excluded(b) => b.as_secs_f64().to_string(),
            Bound::Unbounded => "".to_string(),
        };

        let end_str = match self.end {
            Bound::Included(b) | Bound::Excluded(b) => b.as_secs_f64().to_string(),
            Bound::Unbounded => "".to_string(),
        };
        write!(f, "[{}, {}]", start_str, end_str)
    }
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

// TODO(anand): Can I implement this within argus_derive?
macro_rules! impl_bool_expr {
    ($ty:ty$(, $($arg:ident),* )? ) => {
        impl AnyExpr for $ty {
            fn is_numeric(&self) -> bool {
                false
            }

            fn is_boolean(&self) -> bool {
                true
            }

            fn args(&self) -> Vec<super::ExprRef<'_>> {
                vec![$($( self.$arg.as_ref().into(), )* )*]
            }
        }
    };
    ($ty:ty, [$args:ident]) => {
        impl AnyExpr for $ty {
            fn is_numeric(&self) -> bool {
                false
            }

            fn is_boolean(&self) -> bool {
                true
            }

            fn args(&self) -> Vec<super::ExprRef<'_>> {
                self.$args.iter().map(|arg| arg.into()).collect()
            }
        }
    };
}

/// A `bool` literal
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
pub struct BoolLit(pub bool);

impl_bool_expr!(BoolLit);

/// A `bool` variable
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "{}", name)]
pub struct BoolVar {
    /// Variable name
    pub name: String,
}

impl_bool_expr!(BoolVar);

/// A comparison expression
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "{} {} {}", lhs, op, rhs)]
pub struct Cmp {
    /// The type of comparison
    pub op: Ordering,
    /// The LHS for the comparison
    pub lhs: Box<NumExpr>,
    /// The RHS for the comparison
    pub rhs: Box<NumExpr>,
}

impl_bool_expr!(Cmp, lhs, rhs);

/// Logical negation of an expression
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "!({})", arg)]
pub struct Not {
    /// Expression to be negated
    pub arg: Box<BoolExpr>,
}

impl_bool_expr!(Not, arg);

/// Logical conjunction of a list of expressions
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "({})", r#"args.iter().map(ToString::to_string).join(") && (")"#r)]
pub struct And {
    /// Expressions to be "and"-ed
    pub args: Vec<BoolExpr>,
}

impl_bool_expr!(And, [args]);

/// Logical disjunction of a list of expressions
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "({})", r#"args.iter().map(ToString::to_string).join(") || (")"#r)]
pub struct Or {
    /// Expressions to be "or"-ed
    pub args: Vec<BoolExpr>,
}

impl_bool_expr!(Or, [args]);

/// A temporal next expression
///
/// Checks if the next time point in a signal is `true` or not.
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "X ({})", arg)]
pub struct Next {
    /// Argument for `Next`
    pub arg: Box<BoolExpr>,
}
impl_bool_expr!(Next, arg);

/// Temporal "oracle" expression
///
/// This is equivalent to `steps` number of nested [`Next`](BoolExpr::Next)
/// expressions.
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "X[0,{}]({})", steps, arg)]
pub struct Oracle {
    /// Number of steps to look ahead
    pub steps: usize,
    /// Argument for `Oracle`
    pub arg: Box<BoolExpr>,
}
impl_bool_expr!(Oracle, arg);

/// A temporal always expression
///
/// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0, Unbounded)`:
///   checks if the signal is `true` for all points in a signal.
/// - Otherwise: checks if the signal is `true` for all points within the `interval`.
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "G{}({})", interval, arg)]
pub struct Always {
    /// Argument for `Always`
    pub arg: Box<BoolExpr>,
    /// Interval for the expression
    pub interval: Interval,
}
impl_bool_expr!(Always, arg);

/// A temporal eventually expression
///
/// - If the `interval` is `(Unbounded, Unbounded)` or equivalent to `(0, Unbounded)`:
///   checks if the signal is `true` for some point in a signal.
/// - Otherwise: checks if the signal is `true` for some point within the `interval`.
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "F{}({})", interval, arg)]
pub struct Eventually {
    /// Argument for `Eventually`
    pub arg: Box<BoolExpr>,
    /// Interval for the expression
    pub interval: Interval,
}
impl_bool_expr!(Eventually, arg);

/// A temporal until expression
///
/// Checks if the `lhs` is always `true` for a signal until `rhs` becomes `true`.
#[derive(Clone, Debug, PartialEq, argus_derive::BoolExpr, derive_more::Display)]
#[display(fmt = "({}) U{} ({})", lhs, interval, rhs)]
pub struct Until {
    /// LHS to `lhs Until rhs`
    pub lhs: Box<BoolExpr>,
    /// RHS to `lhs Until rhs`
    pub rhs: Box<BoolExpr>,
    /// Interval for the expression
    pub interval: Interval,
}
impl_bool_expr!(Until, lhs, rhs);
