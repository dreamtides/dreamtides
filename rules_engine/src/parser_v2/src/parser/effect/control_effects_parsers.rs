use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{article, word, words, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((disable_activated_abilities(), gain_control(), put_on_top_of_opponent_deck())).boxed()
}

pub fn gain_control<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["gain", "control", "of"])
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::GainControl { target })
}

pub fn put_on_top_of_opponent_deck<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("put")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["on", "top", "of", "the", "opponent's", "deck"]))
        .map(|target| StandardEffect::PutOnTopOfEnemyDeck { target })
}

pub fn disable_activated_abilities<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["disable", "the", "activated", "abilities", "of"])
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["while", "this", "character", "is", "in", "play"]))
        .map(|target| StandardEffect::DisableActivatedAbilitiesWhileInPlay { target })
}
