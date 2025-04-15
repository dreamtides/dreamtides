use std::fs;

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use parser::ability_parser;

#[derive(Parser)]
#[command(version, about = "Parse ability expressions")]
struct Args {
    #[arg(help = "An ability expression to parse")]
    expression: Option<String>,

    #[arg(short, long, help = "File containing expressions to parse")]
    file: Option<String>,
}

fn parse_expression(input: &str, line_num: Option<usize>) -> Option<String> {
    let input = input.to_lowercase();
    let (result, errs) = ability_parser::parse(&input).into_output_errors();
    if let Some(output) = result.as_ref() {
        ron::ser::to_string_pretty(output, ron::ser::PrettyConfig::default().struct_names(true))
            .ok()
    } else {
        errs.into_iter().for_each(|e| {
            let mut report =
                Report::build(ReportKind::Error, (), e.span().start).with_message(e.to_string());

            if let Some(line) = line_num {
                report = report.with_note(format!("Error in line {}", line));
            }

            report
                .with_label(
                    Label::new(e.span().into_range())
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(&input))
                .unwrap()
        });
        None
    }
}

fn main() {
    let args = Args::parse();

    if let Some(file) = args.file {
        let contents = fs::read_to_string(&file).unwrap();
        let mut results = Vec::new();

        for (i, line) in contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(result) = parse_expression(line, Some(i + 1)) {
                results.push(result);
            } else {
                return;
            }
        }

        println!("[{}]", results.join(",\n"));
    } else if let Some(expr) = args.expression {
        if let Some(result) = parse_expression(&expr, None) {
            println!("{}", result);
        }
    } else {
        println!("No expression or file provided");
    }
}
