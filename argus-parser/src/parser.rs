use chumsky::input::SpannedInput;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::lexer::{Span, Token};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Type {
    #[default]
    Unknown,
    Bool,
    UInt,
    Int,
    Float,
}

impl Type {
    /// Get the lowest common supertype for the given types to a common
    fn get_common_cast(self, other: Self) -> Self {
        use Type::*;
        match (self, other) {
            (Unknown, other) | (other, Unknown) => other,
            (Bool, ty) | (ty, Bool) => ty,
            (UInt, Int) => Float,
            (UInt, Float) => Float,
            (Int, UInt) => Float,
            (Int, Float) => Float,
            (Float, UInt) => Float,
            (Float, Int) => Float,
            (lhs, rhs) => {
                assert_eq!(lhs, rhs);
                rhs
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interval<'src> {
    a: Box<Spanned<Expr<'src>>>,
    b: Box<Spanned<Expr<'src>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOps {
    Neg,
    Not,
    Next,
    Always,
    Eventually,
}

impl UnaryOps {
    fn default_type(&self) -> Type {
        match self {
            UnaryOps::Neg => Type::Float,
            _ => Type::Bool,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOps {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,
    And,
    Or,
    Implies,
    Xor,
    Equiv,
    Until,
}

impl BinaryOps {
    fn default_type(&self) -> Type {
        match self {
            BinaryOps::Add | BinaryOps::Sub | BinaryOps::Mul | BinaryOps::Div => Type::Float,
            _ => Type::Bool,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    Error,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Var {
        name: &'src str,
        kind: Type,
    },
    Unary {
        op: UnaryOps,
        interval: Option<Spanned<Interval<'src>>>,
        arg: Box<Spanned<Self>>,
    },
    Binary {
        op: BinaryOps,
        interval: Option<Spanned<Interval<'src>>>,
        args: (Box<Spanned<Self>>, Box<Spanned<Self>>),
    },
}

impl<'src> Expr<'src> {
    fn get_type(&self) -> Type {
        match self {
            Expr::Error => Type::Unknown,
            Expr::Bool(_) => Type::Bool,
            Expr::Int(_) => Type::Int,
            Expr::UInt(_) => Type::UInt,
            Expr::Float(_) => Type::Float,
            Expr::Var { name: _, kind } => *kind,
            Expr::Unary {
                op,
                interval: _,
                arg: _,
            } => op.default_type(),
            Expr::Binary {
                op,
                interval: _,
                args: _,
            } => op.default_type(),
        }
    }

    /// Make untyped (`Type::Unknown`) expressions into the given type.
    /// Returns a boolean flag to denote successful transformation or not.
    fn make_typed(&mut self, ty: Type) -> bool {
        match self {
            Expr::Var { name: _, kind } => {
                *kind = ty;
                true
            }
            _ => false,
        }
    }

    fn var(name: &'src str) -> Self {
        Self::Var {
            name,
            kind: Type::Unknown,
        }
    }

    fn unary_op(op: UnaryOps, arg: Box<Spanned<Self>>, interval: Option<Spanned<Interval<'src>>>) -> Self {
        let mut arg = arg;
        (*arg).0.make_typed(op.default_type());
        Self::Unary { op, interval, arg }
    }

    fn binary_op(
        op: BinaryOps,
        args: (Box<Spanned<Self>>, Box<Spanned<Self>>),
        interval: Option<Spanned<Interval<'src>>>,
    ) -> Self {
        let mut args = args;

        let lhs = &mut (*args.0).0;
        let rhs = &mut (*args.1).0;

        let common_type = lhs.get_type().get_common_cast(rhs.get_type());
        lhs.make_typed(common_type);
        rhs.make_typed(common_type);

        Self::Binary { op, interval, args }
    }
}

pub type Spanned<T> = (T, Span);

pub type Error<'tokens, 'src> = extra::Err<Rich<'tokens, Token<'src>, Span>>;

// The type of the input that our parser operates on. The input is the `&[(Token,
// Span)]` token buffer generated by the lexer, wrapped in a `SpannedInput` which
// 'splits' it apart into its constituent parts, tokens and spans, for chumsky
// to understand.
type ParserInput<'tokens, 'src> = SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

pub fn num_expr_parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, Error<'tokens, 'src>> + Clone {
    recursive(|num_expr| {
        let var = select! { Token::Ident(ident) => Expr::Var{ name: ident.clone(), kind: Type::default()} }
            .labelled("variable");

        let num_literal = select! {
            Token::Int(val) => Expr::Int(val),
            Token::UInt(val) => Expr::UInt(val),
            Token::Float(val) => Expr::Float(val),
        }
        .labelled("number literal");

        let num_atom = var
            .or(num_literal)
            .map_with_span(|expr, span: Span| (expr, span))
            // Atoms can also just be normal expressions, but surrounded with parentheses
            .or(num_expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
            // Attempt to recover anything that looks like a parenthesised expression but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::LParen,
                Token::RParen,
                [(Token::LBracket, Token::RBracket)],
                |span| (Expr::Error, span),
            )))
            .boxed();

        let neg_op = just(Token::Minus)
            .map_with_span(|_, span: Span| (UnaryOps::Neg, span))
            .repeated()
            .foldr(num_atom, |op, rhs| {
                let span = op.1.start..rhs.1.end;
                (Expr::unary_op(op.0, Box::new(rhs), None), span.into())
            });

        // Product ops (multiply and divide) have equal precedence
        let product_op = {
            let op = just(Token::Times)
                .to(BinaryOps::Mul)
                .or(just(Token::Divide).to(BinaryOps::Div));
            neg_op.clone().foldl(op.then(neg_op).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        // Sum ops (add and subtract) have equal precedence
        let sum_op = {
            let op = just(Token::Plus)
                .to(BinaryOps::Add)
                .or(just(Token::Minus).to(BinaryOps::Sub));
            product_op.clone().foldl(op.then(product_op).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        sum_op.labelled("numeric expression").as_context()
    })
}

pub fn parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, Error<'tokens, 'src>> + Clone {
    let interval = {
        let num_literal = select! {
            Token::Int(val) => Expr::Int(val),
            Token::UInt(val) => Expr::UInt(val),
            Token::Float(val) => Expr::Float(val),
        }
        .map_with_span(|lit, span: Span| (lit, span));
        let sep = just(Token::Comma).or(just(Token::DotDot));

        num_literal
            .then_ignore(sep)
            .then(num_literal)
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|(a, b)| {
                let span = a.1.start..b.1.end;
                (
                    Interval {
                        a: Box::new(a),
                        b: Box::new(b),
                    },
                    span.into(),
                )
            })
    };
    let num_expr = num_expr_parser();

    recursive(|expr| {
        let literal = select! {
            Token::Bool(val) => Expr::Bool(val),
        }
        .labelled("boolean literal");

        let var = select! { Token::Ident(ident) => Expr::Var{ name: ident.clone(), kind: Type::default()} }
            .labelled("variable");

        // Relational ops (<, <=, >, >=) have equal precedence
        let relational_op = {
            let op = just(Token::Lt).to(BinaryOps::Lt).or(just(Token::Le)
                .to(BinaryOps::Le)
                .or(just(Token::Gt).to(BinaryOps::Gt).or(just(Token::Ge).to(BinaryOps::Ge))));
            num_expr.clone().foldl(op.then(num_expr).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        // Equality ops (==, !=) have equal precedence
        let equality_op = {
            let op = just(Token::Eq)
                .to(BinaryOps::Eq)
                .or(just(Token::Neq).to(BinaryOps::Neq));
            relational_op
                .clone()
                .foldl(op.then(relational_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
                })
        }
        .labelled("atomic predicate");

        let atom = equality_op
            .or(var.or(literal).map_with_span(|expr, span| (expr, span)))
            // Atoms can also just be normal expressions, but surrounded with parentheses
            .or(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
            // Attempt to recover anything that looks like a parenthesised expression but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::LParen,
                Token::RParen,
                [],
                |span| (Expr::Error, span),
            )))
            .boxed();

        let not_op = just(Token::Not)
            .map_with_span(|_, span: Span| (UnaryOps::Not, span))
            .repeated()
            .foldr(atom, |op, rhs| {
                let span = op.1.start..rhs.1.end;
                (Expr::unary_op(op.0, Box::new(rhs), None), span.into())
            });

        let next_op = just(Token::Next)
            .map_with_span(|_, span: Span| (UnaryOps::Next, span))
            .then(interval.or_not())
            .repeated()
            .foldr(not_op, |(op, interval), rhs| {
                let span = op.1.start..rhs.1.end;
                (Expr::unary_op(op.0, Box::new(rhs), interval), span.into())
            });

        let unary_temporal_op = {
            let op = just(Token::Eventually)
                .to(UnaryOps::Eventually)
                .or(just(Token::Always).to(UnaryOps::Always));
            op.map_with_span(|op, span: Span| (op, span))
                .then(interval.or_not())
                .repeated()
                .foldr(next_op, |(op, interval), rhs| {
                    let span = op.1.start..rhs.1.end;
                    (Expr::unary_op(op.0, Box::new(rhs), interval), span.into())
                })
        };

        let binary_temporal_op = unary_temporal_op
            .clone()
            .then(just(Token::Until).to(BinaryOps::Until).then(interval.or_not()))
            .repeated()
            .foldr(unary_temporal_op, |(lhs, (op, interval)), rhs| {
                let span = lhs.1.start..rhs.1.end;
                assert_eq!(op, BinaryOps::Until);
                (
                    Expr::binary_op(op, (Box::new(lhs), Box::new(rhs)), interval),
                    span.into(),
                )
            });

        let and_op = {
            let op = just(Token::And).to(BinaryOps::And);
            binary_temporal_op
                .clone()
                .foldl(op.then(binary_temporal_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
                })
        };

        let or_op = {
            let op = just(Token::Or).to(BinaryOps::Or);
            and_op.clone().foldl(op.then(and_op).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        let xor_op = {
            let op = just(Token::Xor).to(BinaryOps::Xor);
            or_op.clone().foldl(op.then(or_op).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        let implies_equiv_op = {
            let op = just(Token::Implies)
                .to(BinaryOps::Implies)
                .or(just(Token::Equiv).to(BinaryOps::Equiv));
            xor_op.clone().foldl(op.then(xor_op).repeated(), |a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::binary_op(op, (Box::new(a), Box::new(b)), None), span.into())
            })
        };

        implies_equiv_op.labelled("expression").as_context()
    })
}
