use ability_data::cost::Cost;
use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::condition_parser;
use crate::parser::effect::{
    card_effect_parsers, game_effects_parsers, resource_effect_parsers, spark_effect_parsers,
};
use crate::parser::parser_helpers::{
    effect_separator, energy, period, word, words, ParserExtra, ParserInput,
};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::parser(),
        game_effects_parsers::parser(),
        resource_effect_parsers::parser(),
        spark_effect_parsers::parser(),
    ))
    .boxed()
}

pub fn effect_or_compound_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone {
    choice((
        optional_effect_with_trigger_cost_parser(),
        optional_effect_parser(),
        conditional_effect_parser(),
        standard_effect_parser(),
    ))
    .boxed()
}

fn optional_effect_with_trigger_cost_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone {
    words(&["you", "may", "pay"])
        .ignore_then(energy())
        .map(|cost| Cost::Energy(Energy(cost)))
        .then_ignore(word("to"))
        .then(single_effect_parser())
        .then_ignore(period())
        .map(|(trigger_cost, effect)| {
            Effect::WithOptions(EffectWithOptions {
                effect,
                optional: true,
                trigger_cost: Some(trigger_cost),
                condition: None,
            })
        })
}

fn conditional_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone {
    condition_parser::condition_parser()
        .then(
            single_effect_parser().separated_by(effect_separator()).at_least(1).collect::<Vec<_>>(),
        )
        .then_ignore(period())
        .map(|(condition, effects)| {
            if effects.len() == 1 {
                Effect::WithOptions(EffectWithOptions {
                    effect: effects.into_iter().next().unwrap(),
                    optional: false,
                    trigger_cost: None,
                    condition: Some(condition),
                })
            } else {
                Effect::List(
                    effects
                        .into_iter()
                        .map(|effect| EffectWithOptions {
                            effect,
                            optional: false,
                            trigger_cost: None,
                            condition: Some(condition.clone()),
                        })
                        .collect(),
                )
            }
        })
}

fn optional_effect_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone
{
    words(&["you", "may"])
        .ignore_then(
            single_effect_parser().separated_by(effect_separator()).at_least(1).collect::<Vec<_>>(),
        )
        .then_ignore(period())
        .map(|effects| {
            if effects.len() == 1 {
                Effect::WithOptions(EffectWithOptions {
                    effect: effects.into_iter().next().unwrap(),
                    optional: true,
                    trigger_cost: None,
                    condition: None,
                })
            } else {
                Effect::List(
                    effects
                        .into_iter()
                        .map(|effect| EffectWithOptions {
                            effect,
                            optional: true,
                            trigger_cost: None,
                            condition: None,
                        })
                        .collect(),
                )
            }
        })
}

fn standard_effect_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone
{
    single_effect_parser()
        .separated_by(effect_separator())
        .at_least(1)
        .collect::<Vec<_>>()
        .then_ignore(period())
        .map(|effects| {
            if effects.len() == 1 {
                Effect::Effect(effects.into_iter().next().unwrap())
            } else {
                Effect::List(effects.into_iter().map(EffectWithOptions::new).collect())
            }
        })
}
