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

use ability_data::cost::Cost;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::determiner_parser;
use crate::parser_utils::{count, numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("$")
            .ignore_then(text::int(10))
            .map(|s: &str| Cost::Energy(Energy(s.parse().unwrap()))),
        numeric("banish", count, "cards from your void").map(Cost::BanishCardsFromYourVoid),
        phrase("abandon").ignore_then(determiner_parser::your_action()).map(Cost::AbandonCharacter),
    ))
    .boxed()
}

/// Alternate phrasing for costs, which are written in static abilities, for
/// example "You may play this event for $0 by abandoning a character".
pub fn inflected_additional_cost<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    phrase("banishing another card from your void").to(Cost::BanishCardsFromYourVoid(1)).boxed()
}
