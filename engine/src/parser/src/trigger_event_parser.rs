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

use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::trigger_event::TriggerEvent;
use chumsky::prelude::choice;
use chumsky::Parser;

use crate::determiner_parser;
use crate::parser_utils::{phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((
        materialize(),
        phrase("you play a character")
            .to(TriggerEvent::Play(Predicate::Your(CardPredicate::Character))),
    ))
}

fn materialize<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you materialize")
        .ignore_then(determiner_parser::parser())
        .map(TriggerEvent::Materialize)
}
