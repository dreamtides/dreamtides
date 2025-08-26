use ability_data::ability::Ability;
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use core_data::initialization_error::{ErrorCode, InitializationError};

use crate::parser_utils::{ErrorType, phrase};
use crate::{
    activated_ability_parser, effect_parser, named_ability_parser, static_ability_parser,
    triggered_ability_parser,
};

/// Takes a string containing card rules text and parses it into a
/// `Vec<[Ability]>` data structure.
///
/// Returns a list of [InitializationError]s if the parsing fails.
pub fn parse(input: &str) -> Result<Vec<Ability>, Vec<InitializationError>> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }
    let input = input.to_lowercase();
    let mut abilities = Vec::new();
    let mut errors = Vec::new();

    for result in parse_string(&input) {
        let (result, errs) = result.into_output_errors();
        if let Some(output) = result {
            abilities.push(output);
        } else {
            for e in errs {
                Report::build(ReportKind::Error, (), e.span().start)
                    .with_message(e.to_string())
                    .with_label(
                        Label::new(e.span().into_range())
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(&input))
                    .expect("Failed to print error");

                errors.push(InitializationError::with_details(
                    ErrorCode::AbilityParsingError,
                    "Failed to parse ability",
                    e.reason().to_string(),
                ));
            }
        }
    }

    if errors.is_empty() { Ok(abilities) } else { Err(errors) }
}

/// Takes a string containing card rules text and parses it into a
/// vector of `ParseResult`s for each ability.
///
/// The provided text must be all lowercase.
pub fn parse_string(text: &str) -> Vec<ParseResult<Ability, Rich<'_, char>>> {
    let mut results = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        results.push(single_ability_parser().parse(line));
    }
    results
}

fn single_ability_parser<'a>() -> impl Parser<'a, &'a str, Ability, ErrorType<'a>> {
    choice((
        triggered_ability_parser::parser().map(Ability::Triggered),
        activated_ability_parser::parser().map(Ability::Activated),
        named_ability_parser::parser().map(Ability::Named),
        effect_parser::event().map(Ability::Event),
        static_ability_parser::parser().then_ignore(phrase(".")).map(Ability::Static),
    ))
    .boxed()
}
