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
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{phrase, ErrorType};

/// Parser for expressions describing the target selected for an effect, for
/// example in "Dissolve an enemy character".
pub fn target_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("that character").to(Predicate::That),
        phrase("that event").to(Predicate::That),
        phrase("another")
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control").or_not())
            .map(Predicate::Another),
        phrase("an enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        choice((phrase("a"), phrase("an")))
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Your),
    ))
    .boxed()
}

/// Parser for expressions where the controller has already been described as
/// the acting party, for example in "Whenever you materialize <predicate>".
pub fn your_action<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        phrase("an").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
        phrase("a").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
    ))
}
