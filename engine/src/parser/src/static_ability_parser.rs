// Copyright (c) dreamcaller 2025-present
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ability_data::static_ability::StaticAbility;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::parser_utils::{numeric, phrase, ErrorType};
use crate::{card_predicate_parser, cost_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    choice((
        disable_enemy_materialized_abilities(),
        once_per_turn_play_from_void(),
        enemy_added_cost_to_play(),
        play_from_void_for_cost(),
    ))
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

fn play_from_void_for_cost<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    numeric("you may play this character from your void for $", Energy, "by")
        .then(cost_parser::inflected_additional_cost())
        .map(|(energy_cost, additional_cost)| StaticAbility::PlayFromVoidForCost {
            energy_cost,
            additional_cost,
        })
}

fn disable_enemy_materialized_abilities<'a>(
) -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    let enemy_characters = choice((phrase("the enemy's characters"), phrase("enemy characters")));
    phrase("disable the \"$materialized\" abilities of")
        .ignore_then(enemy_characters)
        .to(StaticAbility::DisableEnemyMaterializedAbilities)
}
