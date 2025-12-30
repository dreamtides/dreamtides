use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points};

use crate::parser::parser_helpers::{
    article, cards, comma, discards, energy, period, points, word, words, ParserExtra, ParserInput,
};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        draw_then_discard(),
        discard_then_draw(),
        draw_cards(),
        discard_cards(),
        gain_energy(),
        gain_points(),
        return_ally_to_hand(),
        return_enemy_or_ally_to_hand(),
    ))
    .boxed()
}

pub fn draw_cards<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("draw")
        .ignore_then(cards())
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

pub fn return_enemy_or_ally_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article())
        .ignore_then(words(&["enemy", "or", "ally"]))
        .ignore_then(words(&["to", "hand"]))
        .then_ignore(period())
        .map(|_| StandardEffect::ReturnToHand { target: Predicate::Any(CardPredicate::Character) })
}

pub fn return_ally_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article())
        .ignore_then(word("ally"))
        .ignore_then(words(&["to", "hand"]))
        .then_ignore(period())
        .map(|_| StandardEffect::ReturnToHand {
            target: Predicate::Another(CardPredicate::Character),
        })
}

pub fn draw_then_discard<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("draw")
        .ignore_then(cards())
        .then_ignore(comma())
        .then_ignore(word("then"))
        .then(word("discard").ignore_then(discards()))
        .then_ignore(period())
        .map(|(draw_count, discard_count)| StandardEffect::DrawThenDiscard {
            draw_count,
            discard_count,
        })
}

pub fn discard_then_draw<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(discards())
        .then_ignore(comma())
        .then_ignore(word("then"))
        .then(word("draw").ignore_then(cards()))
        .then_ignore(period())
        .map(|(discard_count, draw_count)| StandardEffect::DiscardThenDraw {
            discard_count,
            draw_count,
        })
}
