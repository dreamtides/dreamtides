use ability_data::ability::Ability;
use chumsky::prelude::*;

use crate::parser_utils::{phrase, ErrorType};
use crate::{
    activated_ability_parser, effect_parser, static_ability_parser, triggered_ability_parser,
};

/// Takes a string containing card rules text and parses it into a
/// Vec<[Ability]> data structure.
///
/// The provided text must be all lowercase.
pub fn parse(text: &str) -> ParseResult<Vec<Ability>, Rich<'_, char>> {
    parser().parse(text)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Ability>, ErrorType<'a>> {
    let flavor_text = just("{flavor:").then(none_of("}").repeated()).then(just("}")).padded();
    let reminder_text = just("{reminder:").then(none_of("}").repeated()).then(just("}")).padded();

    let single_ability = choice((
        triggered_ability_parser::parser().map(Ability::Triggered),
        activated_ability_parser::parser().map(Ability::Activated),
        effect_parser::event().map(Ability::Event),
        static_ability_parser::parser().then_ignore(phrase(".")).map(Ability::Static),
    ))
    .then_ignore(reminder_text.or_not());

    single_ability
        .separated_by(phrase("$br"))
        .at_least(1)
        .collect()
        .then_ignore(flavor_text.or_not())
        .then_ignore(end())
}
