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

use ability_data::ability::Ability;
use ability_data::effect::{Effect, GameEffect};
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::trigger_event::TriggerEvent;
use ability_data::triggered_ability::TriggeredAbility;
use chumsky::prelude::*;
use core_data::character_type::CharacterType;
use core_data::numerics::Spark;

/// Takes a string containing card rules text and parses it into an [Ability]
/// data structure.
pub fn parse(text: &str) -> Result<Ability, Vec<Simple<char>>> {
    parser().parse(text)
}

fn parser() -> impl Parser<char, Ability, Error = Simple<char>> {
    trigger_keyword()
        .ignore_then(trigger_event())
        .then_ignore(just(","))
        .then(effect_list())
        .then_ignore(just("."))
        .then_ignore(end())
        .map(|(event, effects)| Ability::Triggered(TriggeredAbility::new(event, effects)))
}

fn trigger_keyword() -> impl Parser<char, (), Error = Simple<char>> {
    text::keyword("Whenever").or(text::keyword("When"))
}

fn trigger_event() -> impl Parser<char, TriggerEvent, Error = Simple<char>> {
    choice((
        just("you materialize another warrior").to(TriggerEvent::Materialize(Predicate::Another(
            CardPredicate::CharacterType(CharacterType::Warrior),
        ))),
        just("you play a character")
            .to(TriggerEvent::Play(Predicate::You(CardPredicate::Character))),
    ))
    .padded()
}

fn effect_list() -> impl Parser<char, Effect, Error = Simple<char>> {
    choice((
        just("this character gains +1 spark")
            .to(Effect::Effect(GameEffect::GainSpark(Predicate::This, Spark(1)))),
        just("draw a card").to(Effect::Effect(GameEffect::DrawCards(1))),
    ))
    .padded()
}
