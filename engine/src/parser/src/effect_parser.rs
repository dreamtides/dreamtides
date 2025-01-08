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

use ability_data::effect::{Effect, EffectList, GameEffect};
use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::{Energy, Spark};

use crate::parser_utils::{count, numeric, phrase, ErrorType};
use crate::{card_predicate_parser, condition_parser, determiner_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    conditional_effect().or(optional_effect()).or(simple_effect())
}

fn optional_effect<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    phrase("you may").ignore_then(base_effect()).map(|game_effect| {
        Effect::EffectList(EffectList {
            effects: vec![game_effect],
            optional: true,
            condition: None,
        })
    })
}

fn simple_effect<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    base_effect().map(Effect::Effect)
}

fn conditional_effect<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    phrase("if")
        .ignore_then(condition_parser::parser())
        .then_ignore(phrase(","))
        .then(choice((optional_effect(), simple_effect())))
        .map(|(condition, effect)| match effect {
            Effect::Effect(game_effect) => Effect::EffectList(EffectList {
                effects: vec![game_effect],
                optional: false,
                condition: Some(condition),
            }),
            Effect::EffectList(mut list) => {
                list.condition = Some(condition);
                Effect::EffectList(list)
            }
        })
}

fn base_effect<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    choice((
        discard_cards(),
        dissolve_character(),
        draw_cards(),
        gain_energy(),
        gain_spark_until_next_main_for_each(),
        gain_spark(),
        gains_aegis_this_turn(),
    ))
}

fn draw_cards<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    phrase("draw")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .map(|count| GameEffect::DrawCards { count })
}

fn gain_spark<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then(numeric("gains +", Spark, "spark"))
        .map(|(predicate, spark)| GameEffect::GainsSpark { target: predicate, gained: spark })
}

fn gain_energy<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    numeric("gain $", Energy, "").map(|energy| GameEffect::GainEnergy { gained: energy })
}

fn gain_spark_until_next_main_for_each<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>>
{
    determiner_parser::target_parser()
        .then(numeric("gains +", Spark, "spark until your next main phase for each"))
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("you control"))
        .map(|((target, spark), counted)| {
            GameEffect::TargetGainsSparkUntilYourNextMainPhaseForEach {
                target,
                gained: spark,
                for_each: Predicate::Your(counted),
            }
        })
}

fn dissolve_character<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    phrase("dissolve")
        .ignore_then(determiner_parser::target_parser())
        .map(|predicate| GameEffect::DissolveCharacter { target: predicate })
}

fn discard_cards<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    phrase("discard")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .map(|count| GameEffect::DiscardCards { count })
}

fn gains_aegis_this_turn<'a>() -> impl Parser<'a, &'a str, GameEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then_ignore(phrase("gains {kw: aegis} this turn"))
        .map(|target| GameEffect::GainsAegisThisTurn { target })
}
