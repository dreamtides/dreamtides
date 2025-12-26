use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::helpers::{integer, period, word, ParserExtra, ParserInput};

pub fn draw_cards<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("draw")
        .ignore_then(integer())
        .then_ignore(period())
        .map(|count| StandardEffect::DrawCards { count })
}

pub fn discard_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(integer())
        .then_ignore(period())
        .map(|count| StandardEffect::DiscardCards { count })
}

pub fn gain_energy<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain")
        .ignore_then(integer())
        .then_ignore(period())
        .map(|n| StandardEffect::GainEnergy { gains: Energy(n) })
}
