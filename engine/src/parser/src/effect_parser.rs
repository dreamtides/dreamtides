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

use ability_data::effect::{Effect, GameEffect};
use chumsky::error::Rich;
use chumsky::prelude::*;
use chumsky::{extra, text, Parser};
use core_data::numerics::Spark;

use crate::determiner_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Effect, extra::Err<Rich<'a, char>>> {
    choice((gain_spark(),)).padded()
}

fn gain_spark<'a>() -> impl Parser<'a, &'a str, Effect, extra::Err<Rich<'a, char>>> {
    determiner_parser::parser()
        .then(
            just("gains +")
                .ignore_then(text::int(10))
                .then_ignore(just(" spark"))
                .map(|s: &str| Spark(s.parse().unwrap())),
        )
        .map(|(predicate, spark)| Effect::Effect(GameEffect::GainSpark(predicate, spark)))
}
