use std::collections::HashMap;
use std::{env, fmt, fs};

use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::prelude::*;

pub type Span = SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Semicolon,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    Bool(bool),
    Num(&'src str),
    Ident(&'src str),
    Minus,
    Plus,
    Times,
    Divide,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,
    Assign,
    Not,
    And,
    Or,
    Implies,
    Xor,
    Equiv,
    Next,
    Always,
    Eventually,
    Until,
}
