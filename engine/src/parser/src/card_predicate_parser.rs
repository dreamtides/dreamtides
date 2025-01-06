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

use ability_data::predicate::{CardPredicate, Operator};
use chumsky::error::Rich;
use chumsky::prelude::{choice, just};
use chumsky::{extra, text, Parser};
use core_data::character_type::CharacterType;
use core_data::numerics::Energy;

pub fn parser<'a>() -> impl Parser<'a, &'a str, CardPredicate, extra::Err<Rich<'a, char>>> {
    choice((
        character_with_cost(),
        character_type().map(CardPredicate::CharacterType),
        just("card").to(CardPredicate::Card),
        just("character").to(CardPredicate::Character),
        just("event").to(CardPredicate::Event),
    ))
    .padded()
}

fn character_with_cost<'a>() -> impl Parser<'a, &'a str, CardPredicate, extra::Err<Rich<'a, char>>>
{
    just("character with cost $").ignore_then(text::int(10)).then_ignore(just(" or less")).map(
        |s: &str| CardPredicate::CharacterWithCost(Operator::OrLess, Energy(s.parse().unwrap())),
    )
}

fn character_type<'a>() -> impl Parser<'a, &'a str, CharacterType, extra::Err<Rich<'a, char>>> {
    choice((
        just("warrior").to(CharacterType::Warrior),
        just("survivor").to(CharacterType::Survivor),
        just("spirit animal").to(CharacterType::SpiritAnimal),
    ))
    .padded()
}
