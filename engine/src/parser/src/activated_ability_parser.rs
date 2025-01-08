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

use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, ErrorType<'a>> {
    let fast_indicator = phrase("$fastactivated")
        .to(Some(ActivatedAbilityOptions { is_fast: true, is_immediate: false, is_multi: false }))
        .or(phrase("$multiactivated").to(Some(ActivatedAbilityOptions {
            is_fast: false,
            is_immediate: false,
            is_multi: true,
        })))
        .or(phrase("$activated").to(None));

    let costs = cost_parser::parser().separated_by(phrase(",")).collect::<Vec<_>>();

    fast_indicator
        .then(costs)
        .then_ignore(phrase(":"))
        .then(effect_parser::effect())
        .map(|((options, costs), effect)| ActivatedAbility { costs, effect, options })
        .boxed()
}
