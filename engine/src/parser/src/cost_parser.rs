use ability_data::cost::Cost;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::determiner_parser;
use crate::parser_utils::{count, numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("$")
            .ignore_then(text::int(10))
            .map(|s: &str| Cost::Energy(Energy(s.parse().unwrap()))),
        numeric("banish", count, "cards from your void").map(Cost::BanishCardsFromYourVoid),
        phrase("abandon").ignore_then(determiner_parser::your_action()).map(Cost::AbandonCharacter),
        phrase("discard your hand").to(Cost::DiscardHand),
    ))
    .boxed()
}

/// Alternate phrasing for costs, which are written in static abilities, for
/// example "You may play this event for $0 by abandoning a character".
pub fn inflected_additional_cost<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    phrase("banishing another card from your void").to(Cost::BanishCardsFromYourVoid(1)).boxed()
}
