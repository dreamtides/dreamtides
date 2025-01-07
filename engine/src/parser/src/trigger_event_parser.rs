use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::trigger_event::{TriggerEvent, TriggerKeyword};
use chumsky::prelude::choice;
use chumsky::{IterParser, Parser};

use crate::determiner_parser;
use crate::parser_utils::{phrase, ErrorType};

pub fn event_parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((
        materialize(),
        phrase("you play a character")
            .to(TriggerEvent::Play(Predicate::Your(CardPredicate::Character))),
    ))
}

pub fn keyword_parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    let single_keyword = choice((
        phrase("$materialized").to(TriggerKeyword::Materialized),
        phrase("$judgment").to(TriggerKeyword::Judgment),
        phrase("$dissolved").to(TriggerKeyword::Dissolved),
    ));

    single_keyword
        .separated_by(phrase(","))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(TriggerEvent::Keywords)
}

fn materialize<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you materialize")
        .ignore_then(determiner_parser::parser())
        .map(TriggerEvent::Materialize)
}
