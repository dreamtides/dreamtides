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
use chumsky::prelude::choice;
use chumsky::Parser;
use core_data::character_type::CharacterType;
use core_data::numerics::Energy;

use crate::parser_utils::{numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    choice((
        character_with_cost(),
        character_type().map(CardPredicate::CharacterType),
        choice((phrase("cards"), phrase("card"))).to(CardPredicate::Card),
        character().to(CardPredicate::Character),
        choice((phrase("events"), phrase("event"))).to(CardPredicate::Event),
    ))
}

fn character_with_cost<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(numeric("with cost $", Energy, "or less"))
        .map(|cost| CardPredicate::CharacterWithCost(cost, Operator::OrLess))
}

fn character_type<'a>() -> impl Parser<'a, &'a str, CharacterType, ErrorType<'a>> {
    phrase("{cardtype: ")
        .ignore_then(choice((
            choice((phrase("warriors"), phrase("warrior"))).to(CharacterType::Warrior),
            choice((phrase("survivors"), phrase("survivor"))).to(CharacterType::Survivor),
            choice((phrase("spirit animals"), phrase("spirit animal")))
                .to(CharacterType::SpiritAnimal),
        )))
        .then_ignore(phrase("}"))
}

fn character<'a>() -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    choice((phrase("characters"), phrase("character")))
}
