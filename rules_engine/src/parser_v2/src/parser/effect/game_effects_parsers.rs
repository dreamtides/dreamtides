use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{directive, foresee_count, period, word, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn foresee<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    foresee_count()
        .then_ignore(period())
        .map(|count| StandardEffect::Foresee { count })
}

pub fn discover<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("discover")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(period())
        .map(|predicate| StandardEffect::Discover { predicate })
}

pub fn counterspell<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("prevent")
        .ignore_then(word("a"))
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(period())
        .map(|target| StandardEffect::Counterspell { target })
}

pub fn dissolve_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve")
        .ignore_then(word("an"))
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(period())
        .map(|target| StandardEffect::DissolveCharacter { target })
}
