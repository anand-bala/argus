use std::env;

use argus::parse_str;
use ariadne::{sources, Color, Label, Report, ReportKind};

fn main() {
    env_logger::init();
    let src = env::args().nth(1).expect("Expected expression");

    match parse_str(&src) {
        Ok(expr) => println!("{}", expr),
        Err(errs) => {
            errs.into_iter().for_each(|e| {
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
                    .eprint(sources([(src.clone(), src.clone())]))
                    .unwrap()
            });
        }
    }
}
