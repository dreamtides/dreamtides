/*
 * Copyright (c) dreamcaller 2025-present
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use ability_data::ability::Ability;
use ability_data::effect::{Effect, EffectList};
use ability_data::trigger_event::TriggerEvent;
use chumsky::prelude::*;

pub fn parser() -> impl Parser<char, Ability, Error = Simple<char>> {
    trigger_keyword()
        .ignore_then(trigger_event())
        .then_ignore(just(","))
        .then(effect_list())
        .then_ignore(just("."))
        .then_ignore(end())
        .map(|(event, effects)| Ability::Triggered(event, effects))
}

fn trigger_keyword() -> impl Parser<char, (), Error = Simple<char>> {
    text::keyword("Whenever").or(text::keyword("When"))
}

fn trigger_event() -> impl Parser<char, TriggerEvent, Error = Simple<char>> {
    choice((
        just("you materialize another warrior").to(TriggerEvent::MaterializeAWarrior),
        just("you play a character").to(TriggerEvent::PlayACharacter),
    ))
    .padded()
}

fn effect_list() -> impl Parser<char, EffectList, Error = Simple<char>> {
    choice((
        just("this character gains +1 spark")
            .to(EffectList::single(Effect::ThisCharacterGainsPlus1Spark)),
        just("draw a card").to(EffectList::single(Effect::DrawCards(1))),
    ))
    .padded()
}
