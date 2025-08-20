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
    let input = input.to_lowercase();
    let (result, errs) = parse_string(&input).into_output_errors();
    if let Some(output) = result {
        Ok(output)
    } else {
        let mut errors = Vec::new();
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

        Err(errors)
    }
}

/// Takes a string containing card rules text and parses it into a
/// Vec<[Ability]> data structure.
///
/// The provided text must be all lowercase.
pub fn parse_string(text: &str) -> ParseResult<Vec<Ability>, Rich<'_, char>> {
    parser().parse(text)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Ability>, ErrorType<'a>> {
    let flavor_text = just("{flavor:").then(none_of("}").repeated()).then(just("}")).padded();
    let reminder_text = just("{reminder:").then(none_of("}").repeated()).then(just("}")).padded();

    let single_ability = choice((
        triggered_ability_parser::parser().map(Ability::Triggered),
        activated_ability_parser::parser().map(Ability::Activated),
        named_ability_parser::parser().map(Ability::Named),
        effect_parser::event().map(Ability::Event),
        static_ability_parser::parser().then_ignore(phrase(".")).map(Ability::Static),
    ))
    .then_ignore(reminder_text.or_not())
    .boxed();

    let ability_block = phrase("{ability}")
        .padded()
        .ignore_then(single_ability.clone())
        .then_ignore(phrase("{end-ability}").padded());

    let multiple_abilities = ability_block.repeated().at_least(1).collect();

    choice((multiple_abilities, single_ability.map(|a| vec![a])))
        .then_ignore(flavor_text.or_not())
        .then_ignore(end())
}
