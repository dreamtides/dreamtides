use std::{env, process};

use ariadne::{Color, Label, Report, ReportKind, Source};
use parser::ability_parser;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: parser_cli <expression>");
        process::exit(0)
    }

    let input = args[1].to_lowercase();
    let (result, errs) = ability_parser::parse(&input).into_output_errors();
    if let Some(output) = result.as_ref() {
        println!(
            "{}",
            ron::ser::to_string_pretty(
                output,
                ron::ser::PrettyConfig::default().struct_names(true)
            )
            .unwrap(),
        );
    }

    errs.into_iter().for_each(|e| {
        Report::build(ReportKind::Error, (), e.span().start)
            .with_message(e.to_string())
            .with_label(
                Label::new(e.span().into_range())
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(&args[1]))
            .unwrap()
    });
}
