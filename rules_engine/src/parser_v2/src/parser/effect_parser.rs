use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::effect::{card_effect_parsers, game_effects_parsers, spark_effect_parsers};
use crate::parser::parser_helpers::{ParserExtra, ParserInput};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::parser(),
        game_effects_parsers::parser(),
        spark_effect_parsers::parser(),
    ))
    .boxed()
}

pub fn effect_or_compound_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> + Clone {
    single_effect_parser()
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|effects| {
            if effects.len() == 1 {
                Effect::Effect(effects.into_iter().next().unwrap())
            } else {
                Effect::List(effects.into_iter().map(EffectWithOptions::new).collect())
            }
        })
        .boxed()
}
