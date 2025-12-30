use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points};

use crate::parser::parser_helpers::{
    article, cards, discards, effect_separator, energy, points, word, words, ParserExtra,
    ParserInput,
};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((draw_cards(), discard_cards(), gain_energy(), gain_points(), return_to_hand())).boxed()
}

pub fn draw_cards<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("draw")
        .ignore_then(cards())
        .then_ignore(effect_separator())
        .map(|count| StandardEffect::DrawCards { count })
}

pub fn discard_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(discards())
        .then_ignore(effect_separator())
        .map(|count| StandardEffect::DiscardCards { count })
}

pub fn gain_energy<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain")
        .ignore_then(energy())
        .then_ignore(effect_separator())
        .map(|n| StandardEffect::GainEnergy { gains: Energy(n) })
}

pub fn gain_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain")
        .ignore_then(points())
        .then_ignore(effect_separator())
        .map(|n| StandardEffect::GainPoints { gains: Points(n) })
}

pub fn return_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article())
        .ignore_then(choice((
            words(&["enemy", "or", "ally"]).to(Predicate::Any(CardPredicate::Character)),
            word("ally").to(Predicate::Another(CardPredicate::Character)),
            word("enemy").to(Predicate::Enemy(CardPredicate::Character)),
        )))
        .then_ignore(words(&["to", "hand"]))
        .then_ignore(effect_separator())
        .map(|target| StandardEffect::ReturnToHand { target })
}
