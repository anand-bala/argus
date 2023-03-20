use std::collections::VecDeque;

use super::{Expr, ExprRef};

/// Iterator that starts from some root [`Expr`] and travels down to it's leaf
/// expressions.
///
/// This essentially implements breadth-first search over the expression tree rooted at
/// the given [`Expr`].
pub struct AstIter<'a> {
    queue: VecDeque<ExprRef<'a>>,
}

impl<'a> AstIter<'a> {
    /// Create an iterator that traverses an [`Expr`] from root to leaf.
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

        let expr: &dyn Expr = match expr_ref {
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
