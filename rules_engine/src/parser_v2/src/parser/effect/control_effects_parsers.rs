use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{article, word, words, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

/// Parses control effect text into [StandardEffect] values.
pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((gain_control(), put_on_top_of_opponent_deck())).boxed()
}

/// Parses "gain control of" effects.
pub fn gain_control<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["gain", "control", "of"])
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::GainControl { target })
}

/// Parses "put <predicate> on top of the opponent's deck" effects.
pub fn put_on_top_of_opponent_deck<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("put")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["on", "top", "of", "the", "opponent's", "deck"]))
        .map(|target| StandardEffect::PutOnTopOfEnemyDeck { target })
}
