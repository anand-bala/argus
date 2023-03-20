use std::{any::Any, collections::HashSet};

mod bool_ops;
mod internal_macros;
pub mod iter;
mod num_ops;
mod traits;

pub use bool_ops::*;
pub use num_ops::*;
pub use traits::*;

use crate::{ArgusResult, Error};

/// All expressions that are numeric
#[derive(Clone, Debug, PartialEq)]
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

impl Expr for NumExpr {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn args(&self) -> Vec<&dyn Expr> {
        match self {
            NumExpr::Neg { arg } => vec![arg.as_ref()],
            NumExpr::Add { args } | NumExpr::Mul { args } => args.iter().map(|arg| arg as &dyn Expr).collect(),
            NumExpr::Div { dividend, divisor } => vec![dividend.as_ref(), divisor.as_ref()],
            _ => vec![],
        }
    }
}

/// Types of comparison operations
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ordering {
    Eq,
    NotEq,
    Less { strict: bool },
    Greater { strict: bool },
}

/// All expressions that are evaluated to be of type `bool`
#[derive(Clone, Debug, PartialEq)]
pub enum BoolExpr {
    BoolLit(bool),
    BoolVar { name: String },
    Cmp { op: Ordering, lhs: Box<NumExpr>, rhs: Box<NumExpr> },
    Not { arg: Box<BoolExpr> },
    And { args: Vec<BoolExpr> },
    Or { args: Vec<BoolExpr> },
}

impl Expr for BoolExpr {
    fn args(&self) -> Vec<&dyn Expr> {
        match self {
            BoolExpr::Cmp { op: _, lhs, rhs } => vec![lhs.as_ref(), rhs.as_ref()],
            BoolExpr::Not { arg } => vec![arg.as_ref()],
            BoolExpr::And { args } | BoolExpr::Or { args } => args.iter().map(|arg| arg as &dyn Expr).collect(),
            _ => vec![],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
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
