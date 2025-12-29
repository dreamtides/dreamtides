use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::effect::{card_effect_parsers, game_effects_parsers, spark_effect_parsers};
use crate::parser::parser_helpers::{ParserExtra, ParserInput};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::draw_cards(),
        card_effect_parsers::draw_literal_cards(),
        card_effect_parsers::discard_cards(),
        card_effect_parsers::gain_energy(),
        card_effect_parsers::gain_points(),
        game_effects_parsers::foresee(),
        game_effects_parsers::discover(),
        game_effects_parsers::counterspell(),
        game_effects_parsers::dissolve_character(),
        spark_effect_parsers::kindle(),
    ))
    .boxed()
}
