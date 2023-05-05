use std::any::Any;

use super::iter::AstIter;
use super::ExprRef;

/// A trait representing expressions
pub trait Expr {
    /// Check if the given expression is a numeric expression
    fn is_numeric(&self) -> bool;
    /// Check if the given expression is a boolean expression
    fn is_boolean(&self) -> bool;
    /// Get the arguments to the current expression.
    ///
    /// If the expression doesn't contain arguments (i.e., it is a leaf expression) then
    /// the vector is empty.
    fn args(&self) -> Vec<ExprRef<'_>>;
    /// Helper function for upcasting to [`std::any::Any`] and then downcasting to a
    /// concrete [`BoolExpr`](crate::expr::BoolExpr) or
    /// [`NumExpr`](crate::expr::NumExpr).
    fn as_any(&self) -> &dyn Any;
    /// An iterator over the AST starting from the current expression.
    fn iter(&self) -> AstIter<'_>;
}

impl dyn Expr {
    /// Convenience method to downcast an expression to a concrete expression node.
    pub fn downcast_expr_ref<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.as_any().downcast_ref::<T>()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::super::{arbitrary, BoolExpr, NumExpr};
    use super::*;

    proptest! {
        #[test]
        fn downcast_expr_bool(bool_expr in arbitrary::bool_expr()) {
            let expr_ref = bool_expr.as_ref() as &dyn Expr;

            let downcast_ref = expr_ref.downcast_expr_ref::<BoolExpr>().unwrap();
            assert_eq!(downcast_ref, bool_expr.as_ref());
        }
    }

    proptest! {
        #[test]
        fn downcast_expr_num(num_expr in arbitrary::num_expr()) {
            let expr_ref = num_expr.as_ref() as &dyn Expr;

            let downcast_ref = expr_ref.downcast_expr_ref::<NumExpr>().unwrap();
            assert_eq!(downcast_ref, num_expr.as_ref());
        }
    }
}
