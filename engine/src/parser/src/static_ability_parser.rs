use ability_data::static_ability::StaticAbility;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::card_predicate_parser;
use crate::parser_utils::{numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    choice((once_per_turn_play_from_void(), enemy_added_cost_to_play()))
}

fn once_per_turn_play_from_void<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    phrase("once per turn, you may play a")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from your void"))
        .map(StaticAbility::OncePerTurnPlayFromVoid)
}

fn enemy_added_cost_to_play<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    phrase("the enemy's")
        .ignore_then(card_predicate_parser::parser())
        .then(numeric("cost an additional $", Energy, "to play"))
        .map(|(predicate, cost)| StaticAbility::EnemyAddedCostToPlay(predicate, cost))
}
