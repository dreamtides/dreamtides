use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points};

use crate::parser::parser_helpers::{
    article, cards, directive, discards, energy, points, top_n_cards, word, words, ParserExtra,
    ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        choice((
            each_player_abandons_characters(),
            each_player_discard_cards(),
            discard_from_opponent_hand(),
        ))
        .boxed(),
        choice((
            draw_cards(),
            discard_cards(),
            gain_energy_for_each(),
            gain_energy(),
            gain_points_for_each(),
        ))
        .boxed(),
        choice((
            gain_points(),
            put_cards_from_deck_into_void(),
            reclaim_from_void(),
            return_from_void_to_hand(),
            return_to_hand(),
        ))
        .boxed(),
    ))
    .boxed()
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

pub fn gain_energy_for_each<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("gain")
        .ignore_then(energy())
        .then_ignore(words(&["for", "each"]))
        .then(predicate_parser::predicate_parser())
        .map(|(gains, for_each)| StandardEffect::GainEnergyForEach {
            gains: Energy(gains),
            for_each,
        })
}

pub fn gain_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("gain").ignore_then(points()).map(|n| StandardEffect::GainPoints { gains: Points(n) })
}

pub fn gain_points_for_each<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("gain")
        .ignore_then(points())
        .then_ignore(words(&["for", "each"]))
        .then(predicate_parser::predicate_parser())
        .map(|(gains, for_each)| StandardEffect::GainPointsForEach {
            gain: Points(gains),
            for_count: QuantityExpression::Matching(for_each),
        })
}

pub fn return_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(choice((words(&["to", "hand"]), words(&["to", "your", "hand"]))).boxed())
        .map(|target| StandardEffect::ReturnToHand { target })
}

pub fn return_from_void_to_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("return")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["from", "your", "void", "to", "your", "hand"]))
        .map(|target| StandardEffect::ReturnFromYourVoidToHand { target })
}

pub fn reclaim_from_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("reclaim")
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::ReturnFromYourVoidToPlay { target })
}

/// Parses effects that move the top cards of your deck into your void.
pub fn put_cards_from_deck_into_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["put", "the"])
        .ignore_then(top_n_cards())
        .then_ignore(words(&["of", "your", "deck", "into", "your", "void"]))
        .map(|count| StandardEffect::PutCardsFromYourDeckIntoVoid { count })
}

pub fn each_player_discard_cards<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["each", "player", "discards"])
        .ignore_then(discards())
        .map(|count| StandardEffect::EachPlayerDiscardCards { count })
}

pub fn each_player_abandons_characters<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["each", "player", "abandons"])
        .ignore_then(article())
        .ignore_then(card_predicate_parser::parser())
        .map(|matching| StandardEffect::EachPlayerAbandonsCharacters { matching, count: 1 })
}

fn discard_from_opponent_hand<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["discard", "a", "chosen"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "the", "opponent's", "hand"]))
        .map(|predicate| StandardEffect::DiscardCardFromEnemyHand { predicate })
}
