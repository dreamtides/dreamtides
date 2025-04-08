use ability_data::quantity_expression::QuantityExpression;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{card_predicate_parser, determiner_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, QuantityExpression, ErrorType<'a>> {
    choice((
        card_predicate_parser::parser()
            .then_ignore(phrase("you have discarded this turn"))
            .map(QuantityExpression::DiscardedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("you have drawn this turn"))
            .map(QuantityExpression::CardsDrawnThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("you have played this turn"))
            .map(QuantityExpression::PlayedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("you abandoned this turn"))
            .map(QuantityExpression::AbandonedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("which dissolved this turn"))
            .map(QuantityExpression::DissolvedThisTurn),
        card_predicate_parser::parser()
            .then_ignore(phrase("abandoned"))
            .map(QuantityExpression::AbandonedThisWay),
        determiner_parser::counted_parser().map(QuantityExpression::Matching),
    ))
    .boxed()
}
