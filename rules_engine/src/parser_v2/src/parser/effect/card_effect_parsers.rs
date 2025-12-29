use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points};

use crate::parser::parser_helpers::{
    cards, discards, energy, literal_number, period, points, word, ParserExtra, ParserInput,
};

pub fn draw_cards<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("draw")
        .ignore_then(cards())
        .then_ignore(period())
        .map(|count| StandardEffect::DrawCards { count })
}

pub fn draw_literal_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("draw")
        .ignore_then(literal_number())
        .then_ignore(choice((word("card"), word("cards"))))
        .then_ignore(period())
        .map(|count| StandardEffect::DrawCards { count })
}

pub fn discard_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(discards())
        .then_ignore(period())
        .map(|count| StandardEffect::DiscardCards { count })
}

pub fn gain_energy<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain")
        .ignore_then(energy())
        .then_ignore(period())
        .map(|n| StandardEffect::GainEnergy { gains: Energy(n) })
}

pub fn gain_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain")
        .ignore_then(points())
        .then_ignore(period())
        .map(|n| StandardEffect::GainPoints { gains: Points(n) })
}
