use ability_data::static_ability::{StandardStaticAbility, StaticAbility};
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{energy, period, word, words, ParserExtra, ParserInput};

/// Parses static abilities that apply continuously.
pub fn static_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StaticAbility, ParserExtra<'a>> + Clone {
    standard_static_ability().map(StaticAbility::StaticAbility)
}

fn standard_static_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((enemy_cards_cost_increase(), your_cards_cost_modification())).boxed()
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
