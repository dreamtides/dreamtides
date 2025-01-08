use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{phrase, ErrorType};

/// Parser for expressions describing the target selected for an effect, for
/// example in "Dissolve an enemy character".
pub fn target_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("this card").to(Predicate::This),
        phrase("that character").to(Predicate::That),
        phrase("that event").to(Predicate::That),
        phrase("that card").to(Predicate::That),
        choice((phrase("another"), phrase("other")))
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control").or_not())
            .map(Predicate::Another),
        phrase("an enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        choice((phrase("a"), phrase("an")))
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Your),
    ))
    .boxed()
}

/// Parser for 'for each' expressions which count entities matching a predicate
pub fn counted_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    phrase("for each")
        .ignore_then(choice((
            phrase("other")
                .ignore_then(card_predicate_parser::parser())
                .then_ignore(phrase("you control"))
                .map(Predicate::Another),
            phrase("enemey").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
            card_predicate_parser::parser().then_ignore(phrase("you control")).map(Predicate::Your),
        )))
        .boxed()
}

/// Parser for expressions where the controller has already been described as
/// the acting party, for example in "Whenever you materialize <predicate>".
pub fn your_action<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        phrase("an").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
        phrase("a").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
    ))
    .boxed()
}
