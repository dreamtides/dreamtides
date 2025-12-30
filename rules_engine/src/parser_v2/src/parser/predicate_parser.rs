use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{directive, word, words, ParserExtra, ParserInput};

pub fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((
        this_parser(),
        enemy_or_ally_parser(),
        enemy_parser(),
        ally_parser(),
        any_fast_card_parser(),
        any_character_parser(),
        any_event_parser(),
        any_card_parser(),
    ))
    .boxed()
}

fn this_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["this", "character"]).to(Predicate::This),
        words(&["this", "event"]).to(Predicate::This),
        words(&["this", "card"]).to(Predicate::This),
    ))
}

fn enemy_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("enemy")
        .ignore_then(card_predicate_parser::parser().or_not())
        .map(|pred| Predicate::Enemy(pred.unwrap_or(CardPredicate::Character)))
}

fn ally_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("ally").to(Predicate::Another(CardPredicate::Character))
}

fn enemy_or_ally_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    words(&["enemy", "or", "ally"]).to(Predicate::Any(CardPredicate::Character))
}

fn any_card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("card").map(|_| Predicate::Any(CardPredicate::Card))
}

fn any_character_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    word("character").map(|_| Predicate::Any(CardPredicate::Character))
}

fn any_event_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("event").map(|_| Predicate::Any(CardPredicate::Event))
}

fn any_fast_card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    directive("fast")
        .ignore_then(card_predicate_parser::parser())
        .map(|target| Predicate::Any(CardPredicate::Fast { target: Box::new(target) }))
}
