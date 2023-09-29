use enum_dispatch::enum_dispatch;

use super::{BoolExpr, ExprRef, NumExpr};

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
