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

/// Takes a string containing card rules text and parses it into an [Ability]
/// data structure.
///
/// The provided text must be all lowercase.
pub fn parse(text: &str) -> ParseResult<Ability, Rich<char>> {
    parser().parse(text)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Ability, ErrorType<'a>> {
    choice((
        triggered_ability_parser::parser().map(Ability::Triggered),
        activated_ability_parser::parser().map(Ability::Activated),
        effect_parser::parser().then_ignore(phrase(".")).map(Ability::Event),
        static_ability_parser::parser().then_ignore(phrase(".")).map(Ability::Static),
    ))
    .then_ignore(end())
}
