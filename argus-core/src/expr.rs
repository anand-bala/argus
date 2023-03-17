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
