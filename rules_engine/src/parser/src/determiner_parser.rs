use ability_data::predicate::Predicate;
use chumsky::Parser;
use chumsky::prelude::*;

use crate::card_predicate_parser;
use crate::parser_utils::{ErrorType, a_or_an, phrase};

/// Parser for expressions describing the target selected for an effect, for
/// example in "Dissolve an enemy character".
pub fn target_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("this card").to(Predicate::This),
        phrase("that character").to(Predicate::That),
        phrase("that event").to(Predicate::That),
        phrase("it").to(Predicate::It),
        phrase("them").to(Predicate::Them),
        phrase("that card").to(Predicate::That),
        choice((phrase("another"), phrase("other")))
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Another),
        a_or_an()
            .ignore_then(phrase("allied"))
            .ignore_then(card_predicate_parser::parser())
            .map(Predicate::Your),
        a_or_an()
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Your),
        a_or_an()
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("in your void"))
            .map(Predicate::YourVoid),
        a_or_an()
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("in the enemy's void"))
            .map(Predicate::EnemyVoid),
        phrase("an enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        phrase("a played enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        phrase("another").ignore_then(card_predicate_parser::parser()).map(Predicate::AnyOther),
        a_or_an().ignore_then(card_predicate_parser::parser()).map(Predicate::Any),
    ))
    .boxed()
}

/// Parser for 'for each' expressions which count entities matching a predicate
pub fn for_each_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    phrase("for each").ignore_then(counted_parser())
}

/// Parser for expressions describing multiple matching objects, such as in
/// "banish two [enemy warriors]"
pub fn counted_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("other")
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you control"))
            .map(Predicate::Another),
        phrase("enemy").ignore_then(card_predicate_parser::parser()).map(Predicate::Enemy),
        phrase("allied").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
        card_predicate_parser::parser().then_ignore(phrase("you control")).map(Predicate::Your),
        card_predicate_parser::parser()
            .then_ignore(phrase("in your void"))
            .map(Predicate::YourVoid),
        card_predicate_parser::parser()
            .then_ignore(phrase("in the enemy's void"))
            .map(Predicate::EnemyVoid),
        card_predicate_parser::parser().map(Predicate::Any),
    ))
}

/// Parser for expressions describing multiple matching objects, when the
/// controler is already implicitly specified, for example in 'Abandon any
/// number of characters.'
pub fn your_action_counted_parser<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("other").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        card_predicate_parser::parser().map(Predicate::Your),
    ))
}

/// Parser for expressions where the controller has already been described as
/// the acting party, for example in "Whenever you materialize <predicate>".
pub fn your_action<'a>() -> impl Parser<'a, &'a str, Predicate, ErrorType<'a>> {
    choice((
        phrase("this character").to(Predicate::This),
        phrase("this event").to(Predicate::This),
        phrase("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another),
        phrase("an allied").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
        phrase("an").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
        phrase("a").ignore_then(card_predicate_parser::parser()).map(Predicate::Your),
    ))
}
