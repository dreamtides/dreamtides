use ability_data::trigger_event::{TriggerEvent, TriggerKeyword};
use chumsky::prelude::choice;
use chumsky::{IterParser, Parser};

use crate::determiner_parser;
use crate::parser_utils::{phrase, ErrorType};

pub fn event_parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((materialize(), play(), discard())).boxed()
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
        .boxed()
}

fn materialize<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you materialize")
        .ignore_then(determiner_parser::your_action())
        .map(TriggerEvent::Materialize)
        .boxed()
}

fn play<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you play").ignore_then(determiner_parser::your_action()).map(TriggerEvent::Play).boxed()
}

fn discard<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you discard")
        .ignore_then(determiner_parser::your_action())
        .map(TriggerEvent::Discard)
        .boxed()
}
