use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("that character").to(Predicate::That),
        phrase("that event").to(Predicate::That),
        phrase("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        phrase("an enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        choice((phrase("a"), phrase("an")))
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Your),
    ))
}
