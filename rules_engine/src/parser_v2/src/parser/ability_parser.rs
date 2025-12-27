use ability_data::ability::{Ability, EventAbility};
use ability_data::effect::Effect;
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
    effect_parser::single_effect_parser().map(|standard_effect| {
        Ability::Event(EventAbility {
            additional_cost: None,
            effect: Effect::Effect(standard_effect),
        })
    })
}
