//! # crate::core logic syntax

use std::time::Duration;

use crate::core::expr::ExprBuilder;
use crate::core::AnyExpr;
mod lexer;
mod syntax;

use chumsky::prelude::Rich;
use itertools::Itertools;
use lexer::{lexer, Token};
use syntax::{parser, Expr, Interval};

use crate::Type;

/// Parse a string expression into a concrete Argus expression.
pub fn parse_str(src: &str) -> Result<crate::core::expr::Expr, Vec<Rich<'_, String>>> {
    use chumsky::prelude::{Input, Parser};

    let (tokens, lex_errors) = lexer().parse(src).into_output_errors();
    log::debug!("** Tokens output **");
    log::debug!("{:#?}", tokens);
    log::debug!("** Lexing Errors: {} **", lex_errors.len());
    log::debug!("\n{}", lex_errors.iter().map(|e| e.to_string()).join("\n"));

    let (parsed, parse_errors) = if let Some(tokens) = &tokens {
        parser()
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors()
    } else {
        (None, Vec::new())
    };

    log::debug!("** Parse output **");
    log::debug!("{:#?}", parsed);
    log::debug!("** Parse Errors: {}**", parse_errors.len());
    log::debug!("\n{}", parse_errors.iter().map(|e| e.to_string()).join("\n"));

    let (expr, expr_errors) = if let Some((ast, span)) = parsed {
        let mut expr_builder = ExprBuilder::new();
        let result = ast_to_expr(&ast, span, &mut expr_builder);
        match result {
            Ok(expr) => (Some(expr), vec![]),
            Err(err) => (None, vec![err]),
        }
    } else {
        (None, vec![])
    };

    log::debug!("** Final Expression **");
    log::debug!("{:#?}", expr);
    log::debug!("** AST to Expr Errors: {} **", expr_errors.len());
    log::debug!("\n{}", expr_errors.iter().map(|e| e.to_string()).join("\n"));

    let errors: Vec<_> = lex_errors
        .into_iter()
        .filter_map(|e| {
            // HACK: Discard empty expected lex errors
            // TODO: not sure why this happens
            use chumsky::error::RichReason::*;
            match e.reason() {
                ExpectedFound { expected, found: _ } if expected.is_empty() => return None,
                _ => {}
            };
            Some(e.map_token(|c| c.to_string()))
        })
        .chain(parse_errors.into_iter().map(|e| e.map_token(|tok| tok.to_string())))
        .chain(expr_errors.into_iter().map(|e| e.map_token(|tok| tok.to_string())))
        .map(|e| e.into_owned())
        .collect();
    log::debug!("** Total Errors: {}**", errors.len());
    log::debug!("[{}]", errors.iter().map(|e| e.to_string()).join("\n"));
    if !errors.is_empty() {
        Err(errors)
    } else {
        expr.ok_or(errors)
    }
}

fn interval_convert(interval: &Interval<'_>) -> crate::core::expr::Interval {
    use core::ops::Bound;
    let a = if let Some(a) = &interval.a {
        match &a.0 {
            Expr::UInt(value) => Bound::Included(Duration::from_secs(*value)),
            Expr::Float(value) => Bound::Included(Duration::from_secs_f64(*value)),
            _ => unreachable!("must be valid numeric literal."),
        }
    } else {
        Bound::Unbounded
    };
    let b = if let Some(b) = &interval.b {
        match &b.0 {
            Expr::UInt(value) => Bound::Included(Duration::from_secs(*value)),
            Expr::Float(value) => Bound::Included(Duration::from_secs_f64(*value)),
            _ => unreachable!("must be valid numeric literal."),
        }
    } else {
        Bound::Unbounded
    };
    crate::core::expr::Interval::new(a, b)
}

/// Convert a parsed [`Expr`] into an [crate::core `Expr`](crate::core::expr::Expr)
fn ast_to_expr<'tokens, 'src: 'tokens>(
    ast: &Expr<'src>,
    span: lexer::Span,
    ctx: &mut ExprBuilder,
) -> Result<crate::core::expr::Expr, Rich<'tokens, Token<'src>, lexer::Span>> {
    match ast {
        Expr::Error => Err(Rich::custom(span, "Errors should have been caught by parser")),
        Expr::Bool(value) => Ok(ctx.bool_const(*value).into()),
        Expr::Int(value) => Ok(ctx.int_const(*value).into()),
        Expr::UInt(value) => Ok(ctx.uint_const(*value).into()),
        Expr::Float(value) => Ok(ctx.float_const(*value).into()),
        Expr::Var { name, kind } => match kind {
            Type::Unknown => Err(Rich::custom(span, "All variables must have defined type by now.")),
            Type::Bool => ctx
                .bool_var(name.to_string())
                .map(|var| var.into())
                .map_err(|err| Rich::custom(span, err.to_string())),
            Type::UInt => ctx
                .uint_var(name.to_string())
                .map(|var| var.into())
                .map_err(|err| Rich::custom(span, err.to_string())),
            Type::Int => ctx
                .int_var(name.to_string())
                .map(|var| var.into())
                .map_err(|err| Rich::custom(span, err.to_string())),
            Type::Float => ctx
                .float_var(name.to_string())
                .map(|var| var.into())
                .map_err(|err| Rich::custom(span, err.to_string())),
        },
        Expr::Unary { op, interval, arg } => {
            let arg = ast_to_expr(&arg.0, arg.1, ctx)?;
            let interval = interval.as_ref().map(|(i, span)| (interval_convert(i), span));
            match op {
                syntax::UnaryOps::Neg => {
                    assert!(interval.is_none());
                    assert!(arg.is_numeric(), "expected numeric expression, got {:?}", arg);
                    let crate::core::expr::Expr::Num(arg) = arg else {
                        unreachable!("- must have numeric expression argument");
                    };
                    Ok(ctx.make_neg(Box::new(arg)).into())
                }
                syntax::UnaryOps::Not => {
                    assert!(interval.is_none());
                    assert!(arg.is_boolean(), "expected boolean expression, got {:?}", arg);
                    let crate::core::expr::Expr::Bool(arg) = arg else {
                        unreachable!("`Not` must have boolean expression argument");
                    };
                    Ok(ctx.make_not(Box::new(arg)).into())
                }
                syntax::UnaryOps::Next => {
                    use core::ops::Bound;
                    assert!(arg.is_boolean(), "expected boolean expression, got {:?}", arg);
                    let crate::core::expr::Expr::Bool(arg) = arg else {
                        unreachable!("`Next` must have boolean expression argument");
                    };
                    match interval {
                        Some((interval, ispan)) => {
                            let steps: usize = match (interval.start, interval.end) {
                                (Bound::Included(start), Bound::Included(end)) => (end - start).as_secs() as usize,
                                _ => {
                                    return Err(Rich::custom(
                                        *ispan,
                                        "Oracle operation (X[..]) cannot have unbounded intervals",
                                    ))
                                }
                            };
                            Ok(ctx.make_oracle(steps, Box::new(arg)).into())
                        }
                        None => Ok(ctx.make_next(Box::new(arg)).into()),
                    }
                }
                syntax::UnaryOps::Always => {
                    assert!(arg.is_boolean(), "expected boolean expression, got {:?}", arg);
                    let crate::core::expr::Expr::Bool(arg) = arg else {
                        unreachable!("`Always` must have boolean expression argument");
                    };
                    match interval {
                        Some((interval, _)) => Ok(ctx.make_timed_always(interval, Box::new(arg)).into()),
                        None => Ok(ctx.make_always(Box::new(arg)).into()),
                    }
                }
                syntax::UnaryOps::Eventually => {
                    assert!(arg.is_boolean(), "expected boolean expression, got {:?}", arg);
                    let crate::core::expr::Expr::Bool(arg) = arg else {
                        unreachable!("`Eventually` must have boolean expression argument");
                    };
                    match interval {
                        Some((interval, _)) => Ok(ctx.make_timed_eventually(interval, Box::new(arg)).into()),
                        None => Ok(ctx.make_eventually(Box::new(arg)).into()),
                    }
                }
            }
        }
        Expr::Binary {
            op,
            interval,
            args: (lhs, rhs),
        } => {
            let lhs = ast_to_expr(&lhs.0, lhs.1, ctx)?;
            let rhs = ast_to_expr(&rhs.0, rhs.1, ctx)?;
            let interval = interval.as_ref().map(|(i, span)| (interval_convert(i), span));

            match op {
                syntax::BinaryOps::Add => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("`Add` must have numeric expression arguments");
                    };
                    ctx.make_add([lhs, rhs])
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Sub => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("`Sub` must have numeric expression arguments");
                    };
                    Ok(ctx.make_sub(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Mul => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("`Mul` must have numeric expression arguments");
                    };
                    ctx.make_mul([lhs, rhs])
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Div => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("`Div` must have numeric expression arguments");
                    };
                    Ok(ctx.make_div(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Lt => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_lt(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Le => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_le(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Gt => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_gt(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Ge => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_ge(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Eq => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got: {}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got: {}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_eq(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::Neq => {
                    assert!(interval.is_none());
                    assert!(lhs.is_numeric(), "expected numeric expression, got {:?}", lhs);
                    assert!(rhs.is_numeric(), "expected numeric expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Num(lhs), crate::core::expr::Expr::Num(rhs)) = (lhs, rhs) else {
                        unreachable!("Relational operation must have numeric expression arguments");
                    };
                    Ok(ctx.make_neq(Box::new(lhs), Box::new(rhs)).into())
                }
                syntax::BinaryOps::And => {
                    assert!(interval.is_none());
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`And` must have boolean expression arguments");
                    };
                    ctx.make_and([lhs, rhs])
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Or => {
                    assert!(interval.is_none());
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`Or` must have boolean expression arguments");
                    };
                    ctx.make_or([lhs, rhs])
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Implies => {
                    assert!(interval.is_none());
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`Implies` must have boolean expression arguments");
                    };
                    ctx.make_implies(Box::new(lhs), Box::new(rhs))
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Equiv => {
                    assert!(interval.is_none());
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`Equiv` must have boolean expression arguments");
                    };
                    ctx.make_equiv(Box::new(lhs), Box::new(rhs))
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Xor => {
                    assert!(interval.is_none());
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`Xor` must have boolean expression arguments");
                    };
                    ctx.make_xor(Box::new(lhs), Box::new(rhs))
                        .map(|ex| ex.into())
                        .map_err(|err| Rich::custom(span, err.to_string()))
                }
                syntax::BinaryOps::Until => {
                    assert!(lhs.is_boolean(), "expected boolean expression, got {:?}", lhs);
                    assert!(rhs.is_boolean(), "expected boolean expression, got {:?}", rhs);

                    let (crate::core::expr::Expr::Bool(lhs), crate::core::expr::Expr::Bool(rhs)) = (lhs, rhs) else {
                        unreachable!("`Until` must have boolean expression arguments");
                    };
                    match interval {
                        Some((interval, _)) => Ok(ctx.make_timed_until(interval, Box::new(lhs), Box::new(rhs)).into()),
                        None => Ok(ctx.make_until(Box::new(lhs), Box::new(rhs)).into()),
                    }
                }
            }
        }
    }
}
