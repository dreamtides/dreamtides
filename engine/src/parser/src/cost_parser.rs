use ability_data::cost::Cost;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::parser_utils::{count, numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("$")
            .ignore_then(text::int(10))
            .map(|s: &str| Cost::Energy(Energy(s.parse().unwrap()))),
        numeric("banish", count, "cards from your void").map(Cost::BanishCardsFromYourVoid),
    ))
}
