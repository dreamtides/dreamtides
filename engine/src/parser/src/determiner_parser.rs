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

use ability_data::predicate::Predicate;
use chumsky::error::Rich;
use chumsky::prelude::{choice, just};
use chumsky::{extra, Parser};

use crate::card_predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Predicate, extra::Err<Rich<'a, char>>> {
    choice((
        just("this character").to(Predicate::This),
        just("this event").to(Predicate::This),
        just("that character").to(Predicate::That),
        just("that event").to(Predicate::That),
        just("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        just("an enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
    ))
    .padded()
}
