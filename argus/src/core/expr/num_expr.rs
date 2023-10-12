//! Numeric expression types
use itertools::Itertools;

use super::{AnyExpr, NumExpr};

// TODO(anand): Can I implement this within argus_derive?
macro_rules! impl_num_expr {
    ($ty:ty$(, $($arg:ident),* )? ) => {
        impl AnyExpr for $ty {
            fn is_numeric(&self) -> bool {
                true
            }

            fn is_boolean(&self) -> bool {
                false
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

/// A signed integer literal
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
pub struct IntLit(pub i64);
impl_num_expr!(IntLit);

/// An unsigned integer literal
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
pub struct UIntLit(pub u64);
impl_num_expr!(UIntLit);

/// A floating point literal
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
pub struct FloatLit(pub f64);
impl_num_expr!(FloatLit);

/// A signed integer variable
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "{}", name)]
pub struct IntVar {
    /// Name of the variable
    pub name: String,
}
impl_num_expr!(IntVar);

/// A unsigned integer variable
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "{}", name)]
pub struct UIntVar {
    /// Name of the variable
    pub name: String,
}
impl_num_expr!(UIntVar);

/// A floating point number variable
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "{}", name)]
pub struct FloatVar {
    /// Name of the variable
    pub name: String,
}
impl_num_expr!(FloatVar);

/// Numeric negation of a numeric expression
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "-({})", arg)]
pub struct Neg {
    /// Numeric expression being negated
    pub arg: Box<NumExpr>,
}
impl_num_expr!(Neg, arg);

/// Arithmetic addition of a list of numeric expressions
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "({})", r#"args.iter().map(ToString::to_string).join(" + ")"#r)]
pub struct Add {
    /// List of expressions being added
    pub args: Vec<NumExpr>,
}
impl_num_expr!(Add, [args]);

/// Subtraction of two numbers
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "({} - {})", lhs, rhs)]
pub struct Sub {
    /// LHS to the expression `lhs - rhs`
    pub lhs: Box<NumExpr>,
    /// RHS to the expression `lhs - rhs`
    pub rhs: Box<NumExpr>,
}
impl_num_expr!(Sub, lhs, rhs);

/// Arithmetic multiplication of a list of numeric expressions
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "({})", r#"args.iter().map(ToString::to_string).join(" * ")"#r)]
pub struct Mul {
    /// List of expressions being multiplied
    pub args: Vec<NumExpr>,
}
impl_num_expr!(Mul, [args]);

/// Divide two expressions `dividend / divisor`
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "({} / {})", dividend, divisor)]
pub struct Div {
    /// The dividend
    pub dividend: Box<NumExpr>,
    /// The divisor
    pub divisor: Box<NumExpr>,
}
impl_num_expr!(Div, dividend, divisor);

/// The absolute value of an expression
#[derive(Clone, Debug, PartialEq, argus_derive::NumExpr, derive_more::Display)]
#[display(fmt = "abs({})", arg)]
pub struct Abs {
    /// Argument to `abs`
    pub arg: Box<NumExpr>,
}
impl_num_expr!(Abs, arg);
