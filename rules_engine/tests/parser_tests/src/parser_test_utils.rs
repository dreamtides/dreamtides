use ability_data::ability::Ability;
use ariadne::{Color, Label, Report, ReportKind, Source};
use parser::ability_parser;

pub fn parse(text: &str) -> Vec<Ability> {
    let input = text.to_lowercase();
    let (result, errs) = ability_parser::parse_string(&input).into_output_errors();

    if !errs.is_empty() {
        for e in errs {
            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new(e.span().into_range())
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(text))
                .unwrap();
        }
        panic!("Error parsing input!");
    }

    result.expect("Error parsing input!")
}
