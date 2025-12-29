use ability_data::ability::{Ability, EventAbility};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{ParserExtra, ParserInput};
use crate::parser::{effect_parser, triggered_parser};

pub fn ability_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Ability, ParserExtra<'a>> + Clone {
    choice((triggered_ability_parser(), event_ability_parser())).boxed()
}

fn triggered_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Ability, ParserExtra<'a>> + Clone {
    triggered_parser::triggered_ability_parser().map(Ability::Triggered)
}

fn event_ability_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Ability, ParserExtra<'a>> + Clone
{
    effect_parser::effect_or_compound_parser()
        .map(|effect| Ability::Event(EventAbility { additional_cost: None, effect }))
}
