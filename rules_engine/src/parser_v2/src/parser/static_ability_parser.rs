use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{
    AlternateCost, StandardStaticAbility, StaticAbility, StaticAbilityWithOptions,
};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    colon, comma, energy, period, spark, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, condition_parser, cost_parser};

/// Parses static abilities that apply continuously.
pub fn static_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StaticAbility, ParserExtra<'a>> + Clone {
    choice((
        standard_static_ability_without_period()
            .then_ignore(word("if"))
            .then(condition_parser::condition_parser())
            .then_ignore(period())
            .map(|(ability, condition)| {
                StaticAbility::WithOptions(StaticAbilityWithOptions {
                    ability,
                    condition: Some(condition),
                })
            }),
        standard_static_ability().map(StaticAbility::StaticAbility),
    ))
    .boxed()
}

fn standard_static_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((
        additional_cost_to_play(),
        abandon_ally_play_character_for_alternate_cost(),
        play_for_alternate_cost(),
        simple_alternate_cost_with_period(),
        allied_spark_bonus(),
        enemy_cards_cost_increase(),
        your_cards_cost_modification(),
    ))
    .boxed()
}

fn standard_static_ability_without_period<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    simple_alternate_cost()
}

fn your_cards_cost_modification<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((your_cards_cost_reduction(), your_cards_cost_increase())).boxed()
}

fn your_cards_cost_reduction<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser()
        .then_ignore(words(&["cost", "you"]))
        .then(energy())
        .then_ignore(word("less"))
        .then_ignore(period())
        .map(|(matching, reduction)| StandardStaticAbility::YourCardsCostReduction {
            matching,
            reduction: Energy(reduction),
        })
}

fn your_cards_cost_increase<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser()
        .then_ignore(words(&["cost", "you"]))
        .then(energy())
        .then_ignore(word("more"))
        .then_ignore(period())
        .map(|(matching, reduction)| StandardStaticAbility::YourCardsCostIncrease {
            matching,
            reduction: Energy(reduction),
        })
}

fn allied_spark_bonus<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    word("allied")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["have", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .then_ignore(period())
        .map(|(matching, added_spark)| StandardStaticAbility::SparkBonusOtherCharacters {
            matching,
            added_spark: Spark(added_spark),
        })
}

fn enemy_cards_cost_increase<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["the", "opponent's"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(word("cost"))
        .then(energy())
        .then_ignore(word("more"))
        .then_ignore(period())
        .map(|(matching, increase)| StandardStaticAbility::EnemyCardsCostIncrease {
            matching,
            increase: Energy(increase),
        })
}

fn abandon_ally_play_character_for_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    cost_parser::abandon_cost_single()
        .then_ignore(colon())
        .then_ignore(words(&["play", "this", "character", "for"]))
        .then(energy())
        .then_ignore(comma())
        .then_ignore(words(&["then", "abandon", "it"]))
        .then_ignore(period())
        .map(|(additional_cost, e)| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost: Energy(e),
                additional_cost: Some(additional_cost),
                if_you_do: Some(Effect::Effect(StandardEffect::PayCost {
                    cost: Cost::AbandonCharactersCount {
                        target: Predicate::This,
                        count: CollectionExpression::Exactly(1),
                    },
                })),
            })
        })
}

fn play_for_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    cost_parser::banish_from_hand_cost()
        .then_ignore(colon())
        .then_ignore(words(&["play", "this", "event", "for"]))
        .then(energy())
        .then_ignore(period())
        .map(|(additional_cost, e)| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost: Energy(e),
                additional_cost: Some(additional_cost),
                if_you_do: None,
            })
        })
}

fn simple_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["this", "event", "costs"]).ignore_then(energy()).map(|e| {
        StandardStaticAbility::PlayForAlternateCost(AlternateCost {
            energy_cost: Energy(e),
            additional_cost: None,
            if_you_do: None,
        })
    })
}

fn simple_alternate_cost_with_period<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["this", "event", "costs"]).ignore_then(energy()).then_ignore(period()).map(|e| {
        StandardStaticAbility::PlayForAlternateCost(AlternateCost {
            energy_cost: Energy(e),
            additional_cost: None,
            if_you_do: None,
        })
    })
}

fn additional_cost_to_play<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["to", "play", "this", "card"])
        .ignore_then(comma())
        .ignore_then(cost_parser::cost_parser())
        .then_ignore(period())
        .map(StandardStaticAbility::AdditionalCostToPlay)
}
