use std::fmt;

use chumsky::prelude::*;

pub type Span = SimpleSpan<usize>;
pub type Output<'a> = Vec<(Token<'a>, Span)>;
pub type Error<'a> = extra::Err<Rich<'a, char, Span>>;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    Equiv,
    Xor,
    Next,
    Always,
    Eventually,
    Until,
}

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Semicolon => write!(f, ";"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Bool(val) => write!(f, "{}", val),
            Token::Num(val) => write!(f, "{}", val),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Minus => write!(f, "-"),
            Token::Plus => write!(f, "+"),
            Token::Times => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Ge => write!(f, ">="),
            Token::Eq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Assign => write!(f, "="),
            Token::Not => write!(f, "!"),
            Token::And => write!(f, "&"),
            Token::Or => write!(f, "|"),
            Token::Implies => write!(f, "->"),
            Token::Equiv => write!(f, "<->"),
            Token::Xor => write!(f, "^"),
            Token::Next => write!(f, "X"),
            Token::Always => write!(f, "G"),
            Token::Eventually => write!(f, "F"),
            Token::Until => write!(f, "U"),
        }
    }
}

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Output<'src>, Error<'src>> {
    // A parser for numbers
    let digits = text::digits(10).slice();

    let frac = just('.').then(digits);

    let exp = just('e').or(just('E')).then(one_of("+-").or_not()).then(digits);

    let number = just('-')
        .or_not()
        .then(text::int(10))
        .then(frac.or_not())
        .then(exp.or_not())
        .map_slice(Token::Num)
        .boxed();

    // A parser for control characters (delimiters, semicolons, etc.)
    let ctrl = choice((
        just(";").to(Token::Semicolon),
        just("[").to(Token::LBracket),
        just("]").to(Token::RBracket),
        just("(").to(Token::LParen),
        just(")").to(Token::RParen),
        just(",").to(Token::Comma),
    ));

    // Lexer for operator symbols
    let op = choice((
        just("<->").to(Token::Equiv),
        just("<=>").to(Token::Equiv),
        just("<=").to(Token::Le),
        just("<").to(Token::Lt),
        just(">=").to(Token::Ge),
        just(">").to(Token::Gt),
        just("!=").to(Token::Neq),
        just("==").to(Token::Eq),
        just("->").to(Token::Implies),
        just("=>").to(Token::Implies),
        just("!").to(Token::Not),
        just("~").to(Token::Not),
        just("\u{00ac}").to(Token::Not), // ¬
        just("&&").to(Token::And),
        just("&").to(Token::And),
        just("\u{2227}").to(Token::And), // ∧
        just("||").to(Token::And),
        just("|").to(Token::And),
        just("\u{2228}").to(Token::Or), // ∨
        just("^").to(Token::Xor),
        just("-").to(Token::Minus),
        just("+").to(Token::Plus),
        just("*").to(Token::Times),
        just("/").to(Token::Divide),
        just("=").to(Token::Assign),
    ));

    // A parser for strings
    // Strings in our grammar are identifiers too
    let quoted_ident = just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .map_slice(Token::Ident);

    // A parser for identifiers and keywords
    let ident = text::ident().map(|ident: &str| match ident {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "G" => Token::Always,
        "alw" => Token::Always,
        "F" => Token::Eventually,
        "ev" => Token::Eventually,
        "X" => Token::Next,
        "U" => Token::Until,
        _ => Token::Ident(ident),
    });

    // A single token can be one of the above
    let token = choice((op, ctrl, quoted_ident, ident, number));

    let comment = just("//").then(any().and_is(just('\n').not()).repeated()).padded();

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        use Token::*;
        let cases = [
            ("true", vec![(Bool(true), Span::new(0, 4))]),
            ("false", vec![(Bool(false), Span::new(0, 5))]),
            (
                "F a",
                vec![(Eventually, Span::new(0, 1)), (Ident("a"), Span::new(2, 3))],
            ),
            (
                "a U b",
                vec![
                    (Ident("a"), Span::new(0, 1)),
                    (Until, Span::new(2, 3)),
                    (Ident("b"), Span::new(4, 5)),
                ],
            ),
        ];

        for (input, expected) in cases {
            let actual = lexer().parse(input).into_result().unwrap();
            assert_eq!(actual, expected);
        }
    }
}
