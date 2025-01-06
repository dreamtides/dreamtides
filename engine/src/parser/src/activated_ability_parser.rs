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

use ability_data::activated_ability::ActivatedAbility;
use chumsky::error::Rich;
use chumsky::prelude::*;
use chumsky::{extra, Parser};

use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, extra::Err<Rich<'a, char>>> {
    just("$activated")
        .padded()
        .ignore_then(cost_parser::parser())
        .then_ignore(just(":").padded())
        .then(effect_parser::parser())
        .then_ignore(just(".").padded())
        .map(|(cost, effect)| ActivatedAbility { cost, effect, options: None })
}
