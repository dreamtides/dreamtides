use std::fs;

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

#[expect(clippy::print_stderr)]
fn parse_expression(input: &str, line_num: Option<usize>) -> Option<String> {
    let input = input.to_lowercase();
    let results = ability_parser::parse(&input);
    match results {
        Ok(output) => ron::ser::to_string_pretty(
            &output,
            ron::ser::PrettyConfig::default().struct_names(true),
        )
        .ok(),
        Err(errs) => {
            for e in errs {
                eprintln!("{:?} at line {}", e, line_num.unwrap_or(0));
            }
            None
        }
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
            println!("{result}");
        }
    } else {
        println!("No expression or file provided");
    }
}
