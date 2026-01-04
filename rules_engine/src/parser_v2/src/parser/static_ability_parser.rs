use ability_data::static_ability::{AlternateCost, StandardStaticAbility, StaticAbility};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    colon, energy, period, spark, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, cost_parser};

/// Parses static abilities that apply continuously.
pub fn static_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StaticAbility, ParserExtra<'a>> + Clone {
    standard_static_ability().map(StaticAbility::StaticAbility)
}

fn standard_static_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((
        play_for_alternate_cost(),
        allied_spark_bonus(),
        enemy_cards_cost_increase(),
        your_cards_cost_modification(),
    ))
    .boxed()
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
