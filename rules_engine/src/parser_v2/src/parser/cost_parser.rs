use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::parser_helpers::{
    article, count_allies, discards, energy, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((abandon_or_discard_cost(), energy_cost(), abandon_cost(), discard_cost())).boxed()
}

fn energy_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    energy().map(|n| Cost::Energy(Energy(n)))
}

fn abandon_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((abandon_cost_with_count(), abandon_cost_single()))
}

fn abandon_or_discard_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    word("abandon")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .filter(|predicate| matches!(predicate, Predicate::Another(CardPredicate::Character)))
        .then_ignore(words(&["or", "discard"]))
        .then_ignore(article())
        .ignore_then(
            predicate_parser::predicate_parser()
                .filter(|predicate| matches!(predicate, Predicate::Any(CardPredicate::Card))),
        )
        .to(Cost::AbandonACharacterOrDiscardACard)
}

fn abandon_cost_with_count<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    word("abandon").ignore_then(count_allies()).map(|count| Cost::AbandonCharactersCount {
        target: Predicate::Another(CardPredicate::Character),
        count: CollectionExpression::Exactly(count),
    })
}

fn abandon_cost_single<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("abandon").ignore_then(article()).ignore_then(predicate_parser::predicate_parser()).map(
        |target| Cost::AbandonCharactersCount { target, count: CollectionExpression::Exactly(1) },
    )
}

fn discard_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(choice((
            discards().map(|count| (CardPredicate::Card, count)),
            article().ignore_then(card_predicate_parser::parser()).map(|predicate| (predicate, 1)),
        )))
        .map(|(predicate, count)| Cost::DiscardCards(predicate, count))
}
