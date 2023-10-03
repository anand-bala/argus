use std::{env, fs};

use argus_parser::lexer;
// use crate::parser::{parser, Error as ParseError};
use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::Parser;

fn main() {
    let src = env::args().nth(1).expect("Expected expression");

    let (tokens, mut errs) = lexer().parse(src.as_str()).into_output_errors();

    println!("*** Outputting tokens ***");
    if let Some(tokens) = &tokens {
        for token in tokens {
            println!("-> {:?}", token);
        }
    }

    let parse_errs = if let Some(tokens) = &tokens {
        let (ast, parse_errs) = parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        println!("*** Outputting tokens ***");
        println!("{:#?}", ast);

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        // .chain(parse_errs.into_iter().map(|e| e.map_token(|tok| tok.to_string())))
        .for_each(|e| {
            Report::build(ReportKind::Error, src.clone(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((src.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((src.clone(), span.into_range()))
                        .with_message(format!("while parsing this {}", label))
                        .with_color(Color::Yellow)
                }))
                .finish()
                .print(sources([(src.clone(), src.clone())]))
                .unwrap()
        });
}
