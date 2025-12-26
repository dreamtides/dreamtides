use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::effect::card_effects;
use crate::parser::helpers::{ParserExtra, ParserInput};

pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((card_effects::draw_cards(), card_effects::discard_cards(), card_effects::gain_energy()))
        .boxed()
}
