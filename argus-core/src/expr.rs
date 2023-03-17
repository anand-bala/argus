/// All expressions that are numeric
#[derive(Clone, Debug)]
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

/// Types of comparison operations
#[derive(Clone, Copy, Debug)]
pub enum Ordering {
    Eq,
    NotEq,
    Less { strict: bool },
    Greater { strict: bool },
}

/// All expressions that are evaluated to be of type `bool`
#[derive(Clone, Debug)]
pub enum BoolExpr {
    BoolLit(bool),
    BoolVar {
        name: String,
    },
    Cmp {
        op: Ordering,
        lhs: Box<NumExpr>,
        rhs: Box<NumExpr>,
    },
    Not {
        arg: Box<BoolExpr>,
    },
    And {
        args: Vec<BoolExpr>,
    },
    Or {
        args: Vec<BoolExpr>,
    },
}

/// Expression builder
#[derive(Clone, Debug)]
pub struct ExprBuilder {}

#[cfg(test)]
pub mod arbitrary {
    //! Helper functions to generate arbitrary expressions using [`proptest`].
    use super::*;
    use proptest::prelude::*;

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
    use super::*;
    use proptest::prelude::*;

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
}
