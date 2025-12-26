use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::effect::card_effect_parsers;
use crate::parser::parser_helpers::{ParserExtra, ParserInput};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::draw_cards(),
        card_effect_parsers::discard_cards(),
        card_effect_parsers::gain_energy(),
    ))
    .boxed()
}
