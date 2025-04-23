use ability_data::ability::EventAbility;
use ability_data::effect::{Effect, EffectWithOptions};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{condition_parser, cost_parser, standard_effect_parser};

pub fn event<'a>() -> impl Parser<'a, &'a str, EventAbility, ErrorType<'a>> {
    cost_parser::parser()
        .then_ignore(just(":"))
        .or_not()
        .then(effect())
        .map(|(additional_cost, effect)| EventAbility { additional_cost, effect })
}

pub fn effect<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    single_effect().repeated().at_least(1).collect::<Vec<_>>().map(|effects| {
        match effects.as_slice() {
            [effect] => effect.clone().to_effect(),
            effects => Effect::List(effects.to_vec()),
        }
    })
}

fn single_effect<'a>() -> impl Parser<'a, &'a str, EffectWithOptions, ErrorType<'a>> {
    conditional_effect()
        .or(optional_effect())
        .or(standard_effect_parser::parser().map(EffectWithOptions::new))
        .then_ignore(choice((just("."), phrase(", then"), phrase("and then"))))
}

fn optional_effect<'a>() -> impl Parser<'a, &'a str, EffectWithOptions, ErrorType<'a>> {
    phrase("you may")
        .ignore_then(
            cost_parser::parser()
                .then_ignore(just("to"))
                .or_not()
                .then(standard_effect_parser::parser()),
        )
        .map(|(maybe_cost, game_effect)| EffectWithOptions {
            effect: game_effect,
            optional: true,
            cost: maybe_cost,
            condition: None,
        })
}

fn conditional_effect<'a>() -> impl Parser<'a, &'a str, EffectWithOptions, ErrorType<'a>> {
    phrase("if")
        .ignore_then(condition_parser::parser())
        .then_ignore(phrase(","))
        .then(choice((
            optional_effect(),
            standard_effect_parser::parser().map(EffectWithOptions::new),
        )))
        .map(|(condition, effect)| effect.with_condition(condition))
}
