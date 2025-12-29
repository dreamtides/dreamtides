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
