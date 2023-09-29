use std::{env, fmt, fs};

use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{input::SpannedInput, prelude::*};

use crate::lexer::{lexer, Span, Token};

pub type Spanned<T> = (T, Span);

// The type of the input that our parser operates on. The input is the `&[(Token, Span)]` token buffer generated by the
// lexer, wrapped in a `SpannedInput` which 'splits' it apart into its constituent parts, tokens and spans, for chumsky
// to understand.
type ParserInput<'tokens, 'src> = SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

pub fn parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
       + Clone {
}