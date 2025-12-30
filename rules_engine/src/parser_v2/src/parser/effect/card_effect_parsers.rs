use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points};

use crate::parser::parser_helpers::{
    article, cards, discards, energy, points, word, words, ParserExtra, ParserInput,
};
use crate::parser::predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((draw_cards(), discard_cards(), gain_energy(), gain_points(), return_to_hand())).boxed()
}

pub fn draw_cards<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("draw").ignore_then(cards()).map(|count| StandardEffect::DrawCards { count })
}

pub fn discard_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("discard").ignore_then(discards()).map(|count| StandardEffect::DiscardCards { count })
}

pub fn gain_energy<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain").ignore_then(energy()).map(|n| StandardEffect::GainEnergy { gains: Energy(n) })
}

pub fn gain_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain").ignore_then(points()).map(|n| StandardEffect::GainPoints { gains: Points(n) })
}

pub fn return_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["to", "hand"]))
        .map(|target| StandardEffect::ReturnToHand { target })
}
