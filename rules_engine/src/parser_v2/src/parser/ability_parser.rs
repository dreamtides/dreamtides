use ability_data::ability::{Ability, EventAbility};
use ability_data::effect::Effect;
use chumsky::prelude::*;

use crate::parser::effect_parser;
use crate::parser::parser_helpers::{ParserExtra, ParserInput};

pub fn ability_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Ability, ParserExtra<'a>> + Clone {
    event_ability_parser().boxed()
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
