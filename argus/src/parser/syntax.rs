use chumsky::input::SpannedInput;
use chumsky::prelude::*;
use chumsky::Parser;

use super::lexer::{Span, Token};
use crate::Type;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Interval<'src> {
    pub a: Option<Box<Spanned<Expr<'src>>>>,
    pub b: Option<Box<Spanned<Expr<'src>>>>,
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
    /// Get the default type for the *arguments* of this kind of expression.
    fn default_args_type(&self) -> Type {
        match self {
            UnaryOps::Neg => Type::Float,
            _ => Type::Bool,
        }
    }

    /// Get the default type the expression with this operator should be
    fn get_default_type(&self) -> Type {
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
    /// Get the default type for the *arguments* of this kind of expression.
    fn default_args_type(&self) -> Type {
        match self {
            BinaryOps::Add
            | BinaryOps::Sub
            | BinaryOps::Mul
            | BinaryOps::Div
            | BinaryOps::Lt
            | BinaryOps::Le
            | BinaryOps::Gt
            | BinaryOps::Ge
            | BinaryOps::Eq
            | BinaryOps::Neq => Type::Float,
            _ => Type::Bool,
        }
    }

    /// Get the default type the expression with this operator should be
    fn get_default_type(&self) -> Type {
        match self {
            BinaryOps::Add | BinaryOps::Sub | BinaryOps::Mul | BinaryOps::Div => Type::Float,
            _ => Type::Bool,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr<'src> {
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
            } => op.get_default_type(),
            Expr::Binary {
                op,
                interval: _,
                args: _,
            } => op.get_default_type(),
        }
    }

    /// Make untyped (`Type::Unknown`) expressions into the given type.
    fn make_typed(mut self, ty: Type) -> Self {
        if let Expr::Var { name: _, kind } = &mut self {
            *kind = ty;
        }
        self
    }

    fn unary_op(op: UnaryOps, arg: Spanned<Self>, interval: Option<Spanned<Interval<'src>>>) -> Self {
        let arg = Box::new((arg.0.make_typed(op.default_args_type()), arg.1));
        Self::Unary { op, interval, arg }
    }

    fn binary_op(
        op: BinaryOps,
        args: (Spanned<Self>, Spanned<Self>),
        interval: Option<Spanned<Interval<'src>>>,
    ) -> Self {
        let (lhs, lspan) = args.0;
        let (rhs, rspan) = args.1;

        let common_type = lhs.get_type().get_common_cast(rhs.get_type());
        let common_type = if Type::Unknown == common_type {
            op.default_args_type()
        } else {
            common_type
        };
        let lhs = Box::new((lhs.make_typed(common_type), lspan));
        let rhs = Box::new((rhs.make_typed(common_type), rspan));

        Self::Binary {
            op,
            interval,
            args: (lhs, rhs),
        }
    }
}

pub type Spanned<T> = (T, Span);

pub type Error<'tokens, 'src> = extra::Err<Rich<'tokens, Token<'src>, Span>>;

// The type of the input that our parser operates on. The input is the `&[(Token,
// Span)]` token buffer generated by the lexer, wrapped in a `SpannedInput` which
// 'splits' it apart into its constituent parts, tokens and spans, for chumsky
// to understand.
type ParserInput<'tokens, 'src> = SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

fn num_expr_parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, Error<'tokens, 'src>> + Clone {
    recursive(|num_expr| {
        let var = select! { Token::Ident(name) => Expr::Var{ name, kind: Type::default()} }.labelled("variable");

        let num_literal = select! {
            Token::Int(val) => Expr::Int(val),
            Token::UInt(val) => Expr::UInt(val),
            Token::Float(val) => Expr::Float(val),
        }
        .labelled("number literal");

        let num_atom = var
            .or(num_literal)
            .map_with(|e, ctx| (e, ctx.span()))
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
            .map_with(|_, e| (UnaryOps::Neg, e.span()))
            .repeated()
            .foldr(num_atom, |op: Spanned<UnaryOps>, rhs: Spanned<Expr<'src>>| {
                let span = op.1.start..rhs.1.end;
                (Expr::unary_op(op.0, rhs, None), span.into())
            });

        // Product ops (multiply and divide) have equal precedence
        let product_op = {
            let op = choice((
                just(Token::Times).to(BinaryOps::Mul),
                just(Token::Divide).to(BinaryOps::Div),
            ));
            neg_op
                .clone()
                .foldl(op.then(neg_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        // Sum ops (add and subtract) have equal precedence
        let sum_op = {
            let op = choice((
                just(Token::Plus).to(BinaryOps::Add),
                just(Token::Minus).to(BinaryOps::Sub),
            ));
            product_op
                .clone()
                .foldl(op.then(product_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        sum_op.labelled("numeric expression").as_context()
    })
}

pub(crate) fn parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, Error<'tokens, 'src>> + Clone {
    let interval = {
        let num_literal = select! {
            Token::UInt(val) => Expr::UInt(val),
            Token::Float(val) => Expr::Float(val),
        }
        .map_with(|lit, e| (lit, e.span()));
        let sep = just(Token::Comma).or(just(Token::DotDot));

        num_literal
            .or_not()
            .then_ignore(sep)
            .then(num_literal.or_not())
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map_with(|(a, b), e| {
                (
                    Interval {
                        a: a.map(Box::new),
                        b: b.map(Box::new),
                    },
                    e.span(),
                )
            })
            .boxed()
    };
    let num_expr = num_expr_parser().boxed();

    recursive(|expr| {
        let literal = select! {
            Token::Bool(val) => Expr::Bool(val),
        }
        .labelled("boolean literal");

        let var =
            select! { Token::Ident(ident) => Expr::Var{ name: ident, kind: Type::default()} }.labelled("variable");

        // Relational ops (<, <=, >, >=, ==, !=) have equal precedence
        let relational_op = {
            let op = choice((
                just(Token::Lt).to(BinaryOps::Lt),
                just(Token::Le).to(BinaryOps::Le),
                just(Token::Gt).to(BinaryOps::Gt),
                just(Token::Ge).to(BinaryOps::Ge),
                just(Token::Eq).to(BinaryOps::Eq),
                just(Token::Neq).to(BinaryOps::Neq),
            ));
            num_expr
                .clone()
                .then(op.then(num_expr))
                .map(|(a, (op, b))| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        }
        .labelled("atomic predicate");

        let atom = relational_op
            .or(var.or(literal).map_with(|expr, e| (expr, e.span())))
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

        let unary_op = {
            let op = choice((
                just(Token::Not).to(UnaryOps::Not),
                just(Token::Next).to(UnaryOps::Next),
                just(Token::Eventually).to(UnaryOps::Eventually),
                just(Token::Always).to(UnaryOps::Always),
            ));
            op.map_with(|op, e| (op, e.span()))
                .then(interval.clone().or_not())
                .try_map(|(op, interval), _| match (op, interval) {
                    ((UnaryOps::Not, _), Some((_, s))) => {
                        Err(Rich::custom(s, "Not (`!`) operator cannot have an interval"))
                    }
                    (o, i) => Ok((o, i)),
                })
                .repeated()
                .foldr(atom, |(op, interval), rhs| {
                    let span = op.1.start..rhs.1.end;
                    (Expr::unary_op(op.0, rhs, interval), span.into())
                })
                .boxed()
        };

        let until_op = unary_op
            .clone()
            .then(just(Token::Until).to(BinaryOps::Until).then(interval.or_not()))
            .repeated()
            .foldr(unary_op, |(lhs, (op, interval)), rhs| {
                let span = lhs.1.start..rhs.1.end;
                assert_eq!(op, BinaryOps::Until);
                (Expr::binary_op(op, (lhs, rhs), interval), span.into())
            })
            .boxed();

        let and_op = {
            let op = just(Token::And).to(BinaryOps::And);
            until_op
                .clone()
                .foldl(op.then(until_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        let or_op = {
            let op = just(Token::Or).to(BinaryOps::Or);
            and_op
                .clone()
                .foldl(op.then(and_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        let xor_op = {
            let op = just(Token::Xor).to(BinaryOps::Xor);
            or_op
                .clone()
                .foldl(op.then(or_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        let implies_equiv_op = {
            let op = just(Token::Implies)
                .to(BinaryOps::Implies)
                .or(just(Token::Equiv).to(BinaryOps::Equiv));
            xor_op
                .clone()
                .foldl(op.then(xor_op).repeated(), |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::binary_op(op, (a, b), None), span.into())
                })
                .boxed()
        };

        implies_equiv_op
            .map(|(expr, span)| (expr.make_typed(Type::Bool), span))
            .labelled("boolean expression")
            .as_context()
    })
}
