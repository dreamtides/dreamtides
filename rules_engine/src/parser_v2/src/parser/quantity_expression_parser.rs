use ability_data::predicate::CardPredicate;
use ability_data::quantity_expression_data::QuantityExpression;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{directive, word, words, ParserExtra, ParserInput};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, QuantityExpression, ParserExtra<'a>> + Clone
{
    card_predicate_parser::parser()
        .then_ignore(words(&["you", "have", "played", "this", "turn"]))
        .map(QuantityExpression::PlayedThisTurn)
        .or(directive("energy-symbol")
            .then_ignore(word("spent"))
            .to(QuantityExpression::ForEachEnergySpentOnThisCard))
        .or(words(&["ally", "abandoned", "this", "turn"])
            .to(())
            .map(|_| QuantityExpression::AbandonedThisTurn(CardPredicate::Character)))
        .or(words(&["ally", "abandoned"])
            .to(())
            .map(|_| QuantityExpression::AbandonedThisWay(CardPredicate::Character)))
        .or(words(&["ally", "returned"])
            .to(())
            .map(|_| QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character)))
        .or(card_predicate_parser::parser()
            .then_ignore(word("abandoned"))
            .map(QuantityExpression::AbandonedThisWay))
        .or(card_predicate_parser::parser()
            .then_ignore(word("returned"))
            .map(QuantityExpression::ReturnedToHandThisWay))
        .or(predicate_parser::predicate_parser().map(QuantityExpression::Matching))
        .boxed()
}
