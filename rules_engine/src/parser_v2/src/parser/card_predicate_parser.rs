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
            base.clone()
                .then(predicate_suffix_parser::with_cost_compared_to_controlled_suffix())
                .map(|(target, (cost_operator, count_matching))| {
                    CardPredicate::CharacterWithCostComparedToControlled {
                        target: Box::new(target),
                        cost_operator,
                        count_matching: Box::new(count_matching),
                    }
                }),
            base.clone()
                .then(predicate_suffix_parser::with_cost_compared_to_void_count_suffix())
                .map(|(target, cost_operator)| {
                    CardPredicate::CharacterWithCostComparedToVoidCount {
                        target: Box::new(target),
                        cost_operator,
                    }
                }),
            base.clone()
                .then(predicate_suffix_parser::with_spark_compared_to_abandoned_suffix())
                .map(|(target, spark_operator)| {
                    CardPredicate::CharacterWithSparkComparedToAbandoned {
                        target: Box::new(target),
                        spark_operator,
                    }
                }),
            base.clone()
                .then(predicate_suffix_parser::with_spark_compared_to_energy_spent_suffix())
                .map(|(target, spark_operator)| {
                    CardPredicate::CharacterWithSparkComparedToEnergySpent {
                        target: Box::new(target),
                        spark_operator,
                    }
                }),
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
            base.clone()
                .then(predicate_suffix_parser::with_materialized_ability_suffix())
                .map(|_| CardPredicate::CharacterWithMaterializedAbility),
            base.clone()
                .then(predicate_suffix_parser::with_activated_ability_suffix())
                .map(|_| CardPredicate::CharacterWithMultiActivatedAbility),
            predicate_suffix_parser::with_cost_compared_to_controlled_suffix().map(
                |(cost_operator, count_matching)| {
                    CardPredicate::CharacterWithCostComparedToControlled {
                        target: Box::new(CardPredicate::Character),
                        cost_operator,
                        count_matching: Box::new(count_matching),
                    }
                },
            ),
            predicate_suffix_parser::with_cost_compared_to_void_count_suffix().map(
                |cost_operator| CardPredicate::CharacterWithCostComparedToVoidCount {
                    target: Box::new(CardPredicate::Character),
                    cost_operator,
                },
            ),
            predicate_suffix_parser::with_spark_compared_to_abandoned_suffix().map(
                |spark_operator| CardPredicate::CharacterWithSparkComparedToAbandoned {
                    target: Box::new(CardPredicate::Character),
                    spark_operator,
                },
            ),
            predicate_suffix_parser::with_spark_compared_to_energy_spent_suffix().map(
                |spark_operator| CardPredicate::CharacterWithSparkComparedToEnergySpent {
                    target: Box::new(CardPredicate::Character),
                    spark_operator,
                },
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
    choice((word("card"), word("cards"))).to(CardPredicate::Card)
}

fn character_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone
{
    choice((word("character"), word("characters"))).to(CardPredicate::Character)
}

fn event_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((word("event"), word("events"))).to(CardPredicate::Event)
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
