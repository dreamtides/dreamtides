use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::parser_helpers::{article, reclaim_cost, word, words, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        gains_reclaim_for_cost(),
        disable_activated_abilities(),
        gain_control(),
        put_on_top_of_opponent_deck(),
    ))
    .boxed()
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

pub fn gains_reclaim_for_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("it")
        .ignore_then(word("gains"))
        .ignore_then(reclaim_cost())
        .then_ignore(words(&["this", "turn"]))
        .map(|cost| StandardEffect::GainsReclaimUntilEndOfTurn {
            target: Predicate::It,
            cost: Some(Energy(cost)),
        })
}
