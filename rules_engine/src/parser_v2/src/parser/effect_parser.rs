use ability_data::cost::Cost;
use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::predicate::CardPredicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::triggered_ability::{TriggeredAbility, TriggeredAbilityOptions};
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::effect::{
    card_effect_parsers, control_effects_parsers, game_effects_parsers, resource_effect_parsers,
    spark_effect_parsers,
};
use crate::parser::parser_helpers::{
    article, comma, discards, effect_separator, energy, period, word, words, ParserExtra,
    ParserInput,
};
use crate::parser::{card_predicate_parser, condition_parser, trigger_parser};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((create_trigger_until_end_of_turn(), base_single_effect_parser())).boxed()
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

fn base_single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::parser(),
        control_effects_parsers::parser(),
        game_effects_parsers::parser(),
        resource_effect_parsers::parser(),
        spark_effect_parsers::parser(),
    ))
    .boxed()
}

fn create_trigger_until_end_of_turn<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["until", "end", "of", "turn"])
        .ignore_then(comma())
        .ignore_then(trigger_parser::trigger_event_parser())
        .then(base_single_effect_parser())
        .map(|(trigger, effect)| StandardEffect::CreateTriggerUntilEndOfTurn {
            trigger: Box::new(TriggeredAbility {
                trigger,
                effect: Effect::Effect(effect),
                options: Some(TriggeredAbilityOptions {
                    once_per_turn: false,
                    until_end_of_turn: true,
                }),
            }),
        })
}

fn optional_effect_with_trigger_cost_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone {
    words(&["you", "may"])
        .ignore_then(trigger_cost_parser())
        .then_ignore(word("to"))
        .then(
            single_effect_parser().separated_by(effect_separator()).at_least(1).collect::<Vec<_>>(),
        )
        .then_ignore(period())
        .map(|(trigger_cost, effects)| {
            if effects.len() == 1 {
                Effect::WithOptions(EffectWithOptions {
                    effect: effects.into_iter().next().unwrap(),
                    optional: true,
                    trigger_cost: Some(trigger_cost),
                    condition: None,
                })
            } else {
                Effect::List(
                    effects
                        .into_iter()
                        .map(|effect| EffectWithOptions {
                            effect,
                            optional: true,
                            trigger_cost: Some(trigger_cost.clone()),
                            condition: None,
                        })
                        .collect(),
                )
            }
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

fn trigger_cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((pay_energy_cost(), discard_cost())).boxed()
}

fn pay_energy_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("pay").ignore_then(energy()).map(|cost| Cost::Energy(Energy(cost)))
}

fn discard_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(choice((
            discards().map(|count| (CardPredicate::Card, count)),
            article().ignore_then(card_predicate_parser::parser()).map(|predicate| (predicate, 1)),
        )))
        .map(|(predicate, count)| Cost::DiscardCards(predicate, count))
}
