use std::any::Any;

use super::iter::AstIter;

/// A trait representing expressions
pub trait Expr {
    fn args(&self) -> Vec<&dyn Expr>;

    fn as_any(&self) -> &dyn Any;

    fn iter(&self) -> AstIter<'_>
    where
        Self: Sized,
    {
        AstIter::new(self)
    }
}

impl dyn Expr {
    pub fn downcast_expr_ref<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.as_any().downcast_ref::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{arbitrary, BoolExpr, NumExpr};
    use super::*;
    use proptest::prelude::*;

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
