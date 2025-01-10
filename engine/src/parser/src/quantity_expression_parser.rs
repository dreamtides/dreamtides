use ability_data::quantity_expression::QuantityExpression;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::card_predicate_parser;
use crate::parser_utils::{phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, QuantityExpression, ErrorType<'a>> {
    choice((
        card_predicate_parser::parser()
            .then_ignore(phrase("you have discarded this turn"))
            .map(QuantityExpression::DiscardedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("you have drawn this turn"))
            .map(QuantityExpression::CardsDrawnThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("in your void"))
            .map(QuantityExpression::CardsInYourVoid),
        card_predicate_parser::parser()
            .then_ignore(phrase("you have played this turn"))
            .map(QuantityExpression::PlayedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("abandoned"))
            .map(QuantityExpression::CharacterAbandoned),
    ))
}
