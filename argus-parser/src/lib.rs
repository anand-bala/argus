//! # Argus logic syntax

mod lexer;
mod parser;

pub use lexer::{lexer, Error as LexError, Span, Token};
pub use parser::{parser, Expr, Interval};
