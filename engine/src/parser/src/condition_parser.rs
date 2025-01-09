use ability_data::condition::Condition;
use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{count, numeric, phrase, ErrorType};

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
        phrase("a")
            .ignore_then(card_predicate_parser::parser())
            .then_ignore(phrase("you controlled dissolved this turn"))
            .map(|predicate| Condition::DissolvedThisTurn {
                predicate: Predicate::Your(predicate),
            }),
        choice((
            phrase("you have discarded a card this turn")
                .to(Condition::CardsDiscardedThisTurn { count: 1 }),
            numeric("you have discarded", count, "cards this turn")
                .map(|count| Condition::CardsDiscardedThisTurn { count }),
        )),
    ))
}
