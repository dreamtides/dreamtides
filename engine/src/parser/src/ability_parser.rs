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
use chumsky::prelude::*;

use crate::parser_utils::{phrase, ErrorType};
use crate::{
    activated_ability_parser, effect_parser, static_ability_parser, triggered_ability_parser,
};

/// Takes a string containing card rules text and parses it into a
/// Vec<[Ability]> data structure.
///
/// The provided text must be all lowercase.
pub fn parse(text: &str) -> ParseResult<Vec<Ability>, Rich<char>> {
    parser().parse(text)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Ability>, ErrorType<'a>> {
    // Parser for flavor text that we'll ignore
    let flavor_text = just("{flavor:").then(none_of("}").repeated()).then(just("}")).padded();

    // Parser for reminder text that we'll ignore
    let reminder_text = just("{reminder:").then(none_of("}").repeated()).then(just("}")).padded();

    let single_ability = choice((
        triggered_ability_parser::parser().map(Ability::Triggered),
        activated_ability_parser::parser().map(Ability::Activated),
        effect_parser::parser().then_ignore(phrase(".")).map(Ability::Event),
        static_ability_parser::parser().then_ignore(phrase(".")).map(Ability::Static),
    ))
    .then_ignore(reminder_text.or_not());

    single_ability
        .separated_by(just("$br"))
        .at_least(1)
        .collect()
        .then_ignore(flavor_text.or_not())
        .then_ignore(end())
}
