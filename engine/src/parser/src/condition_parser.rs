use ability_data::condition::Condition;
use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{count, numeric, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Condition, ErrorType<'a>> {
    choice((
        numeric("you control", count, "other").then(card_predicate_parser::parser()).map(
            |(count, predicate)| Condition::PredicateCount {
                count,
                predicate: Predicate::Your(predicate),
            },
        ),
        numeric("you have", count, "or more cards in your void")
            .map(|count| Condition::CardsInVoidCount { count }),
    ))
}
