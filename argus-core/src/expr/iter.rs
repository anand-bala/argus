//! Iterators for expression trees

use std::collections::VecDeque;

use super::{AnyExpr, ExprRef};

/// Iterator that starts from some root [`AnyExpr`] and travels down to it's leaf
/// expressions.
///
/// This essentially implements breadth-first search over the expression tree rooted at
/// the given [`AnyExpr`].
pub struct AstIter<'a> {
    queue: VecDeque<ExprRef<'a>>,
}

impl<'a> AstIter<'a> {
    /// Create an iterator that traverses an [`AnyExpr`] from root to leaf.
    pub fn new(root: ExprRef<'a>) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(root);
        Self { queue }
    }
}

impl<'a> Iterator for AstIter<'a> {
    type Item = ExprRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let expr_ref = self.queue.pop_front()?;

        let expr: &dyn AnyExpr = match expr_ref {
            ExprRef::Bool(expr) => expr,
            ExprRef::Num(expr) => expr,
        };

        // We need to get all the arguments of the current expression (not including
        //    any intervals), and push them into the queue.
        for arg in expr.args().into_iter() {
            self.queue.push_back(arg);
        }
        // 4. Give the user their expr
        Some(expr_ref)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::expr::{ExprBuilder, ExprRef};

    #[test]
    fn simple_iter() {
        let mut ctx = ExprBuilder::new();

        let x = Box::new(ctx.float_var("x".to_owned()).unwrap());
        let y = Box::new(ctx.float_var("y".to_owned()).unwrap());
        let lit = Box::new(ctx.float_const(2.0));

        let pred1 = Box::new(ctx.make_le(x.clone(), lit.clone()));
        let pred2 = Box::new(ctx.make_gt(y.clone(), lit.clone()));
        let spec = Box::new(ctx.make_or([*pred1.clone(), *pred2.clone()]).unwrap());

        drop(ctx);

        let expr_tree = spec.iter();
        let expected: Vec<ExprRef<'_>> = vec![
            spec.as_ref().into(),
            pred1.as_ref().into(),
            pred2.as_ref().into(),
            x.as_ref().into(),
            lit.as_ref().into(),
            y.as_ref().into(),
            lit.as_ref().into(),
        ];

        for (lhs, rhs) in expr_tree.zip_eq(expected.into_iter()) {
            match (lhs, rhs) {
                (ExprRef::Bool(lhs), ExprRef::Bool(rhs)) => assert_eq!(lhs, rhs),
                (ExprRef::Num(lhs), ExprRef::Num(rhs)) => assert_eq!(lhs, rhs),
                e => panic!("got mismatched pair: {:?}", e),
            }
        }
    }
}
