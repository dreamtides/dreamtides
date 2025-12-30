use ability_data::predicate::CardPredicate;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{directive, subtype, word, ParserExtra, ParserInput};
use crate::parser::predicate_suffix_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    recursive(|parser| {
        let base = choice((
            fast_parser(parser.clone()),
            subtype_parser(),
            character_parser(),
            event_parser(),
            card_parser(),
        ));

        choice((
            base.clone().then(predicate_suffix_parser::with_cost_suffix()).map(
                |(target, (cost, op))| CardPredicate::CardWithCost {
                    target: Box::new(target),
                    cost_operator: op,
                    cost: Energy(cost),
                },
            ),
            base.clone().then(predicate_suffix_parser::with_spark_suffix()).map(
                |(_, (spark_value, op))| CardPredicate::CharacterWithSpark(Spark(spark_value), op),
            ),
            predicate_suffix_parser::with_cost_suffix().map(|(cost, op)| {
                CardPredicate::CardWithCost {
                    target: Box::new(CardPredicate::Character),
                    cost_operator: op,
                    cost: Energy(cost),
                }
            }),
            predicate_suffix_parser::with_spark_suffix()
                .map(|(spark_value, op)| CardPredicate::CharacterWithSpark(Spark(spark_value), op)),
            base,
        ))
    })
    .boxed()
}

fn card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    word("card").to(CardPredicate::Card)
}

fn character_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone
{
    word("character").to(CardPredicate::Character)
}

fn event_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    word("event").to(CardPredicate::Event)
}

fn fast_parser<'a>(
    inner: impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    directive("fast")
        .ignore_then(inner)
        .map(|target| CardPredicate::Fast { target: Box::new(target) })
}

fn subtype_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone
{
    subtype().map(CardPredicate::CharacterType)
}
