use ability_data::predicate::{CardPredicate, Operator};
use chumsky::prelude::choice;
use chumsky::Parser;
use core_data::character_type::CharacterType;
use core_data::numerics::{Energy, Spark};

use crate::parser_utils::{numeric, phrase, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    choice((
        card_with_cost(),
        character_with_cost_compared_to_controlled(),
        character_with_cost_compared_to_abandoned(),
        character_with_spark_compared_to_abandoned_this_turn(),
        character_with_spark_compared_to_abandoned(),
        fast_card(),
        non_recursive_predicate(),
    ))
    .boxed()
}

fn non_recursive_predicate<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    choice((
        character_with_spark(),
        character_with_materialized_ability(),
        character_type().map(CardPredicate::CharacterType),
        choice((phrase("cards"), phrase("card"))).to(CardPredicate::Card),
        character().to(CardPredicate::Character),
        choice((phrase("events"), phrase("event"))).to(CardPredicate::Event),
        choice((phrase("dreams"), phrase("dream"))).to(CardPredicate::Dream),
    ))
    .boxed()
}

fn card_with_cost<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then(numeric("with cost $", Energy, ""))
        .then(choice((
            phrase("or less").to(Operator::OrLess),
            phrase("or more").to(Operator::OrMore),
        )))
        .map(|((target, cost), operator)| CardPredicate::CardWithCost {
            target: Box::new(target),
            cost_operator: operator,
            cost,
        })
        .boxed()
}

fn character_with_spark<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(numeric("with spark", Spark, ""))
        .then(choice((
            phrase("or less").to(Operator::OrLess),
            phrase("or more").to(Operator::OrMore),
        )))
        .map(|(spark, operator)| CardPredicate::CharacterWithSpark(spark, operator))
        .boxed()
}

fn character_with_cost_compared_to_controlled<'a>(
) -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then_ignore(phrase("with cost"))
        .then(choice((
            phrase("less than or equal to").to(Operator::OrLess),
            phrase("equal to").to(Operator::Exactly),
            phrase("greater than or equal to").to(Operator::OrMore),
        )))
        .then(
            phrase("the number of")
                .ignore_then(non_recursive_predicate())
                .then_ignore(phrase("you control")),
        )
        .map(|((target, cost_operator), count_matching)| {
            CardPredicate::CharacterWithCostComparedToControlled {
                target: Box::new(target),
                cost_operator,
                count_matching: Box::new(count_matching),
            }
        })
        .boxed()
}

fn character_with_cost_compared_to_abandoned<'a>(
) -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then_ignore(phrase("with cost"))
        .then(choice((
            numeric("$", Energy, "higher than").map(Operator::HigherBy),
            numeric("$", Energy, "lower than").map(Operator::LowerBy),
            phrase("greater than").to(Operator::OrMore),
            phrase("less than").to(Operator::OrLess),
            phrase("equal to").to(Operator::Exactly),
        )))
        .then_ignore(phrase("the abandoned character"))
        .map(|(target, cost_operator)| CardPredicate::CharacterWithCostComparedToAbandoned {
            target: Box::new(target),
            cost_operator,
        })
        .boxed()
}

fn character_with_materialized_ability<'a>(
) -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(phrase("with a $materialized ability"))
        .to(CardPredicate::CharacterWithMaterializedAbility)
        .boxed()
}

fn character_type<'a>() -> impl Parser<'a, &'a str, CharacterType, ErrorType<'a>> {
    phrase("{cardtype: ")
        .ignore_then(choice((
            choice((phrase("warriors"), phrase("warrior"))).to(CharacterType::Warrior),
            choice((phrase("survivors"), phrase("survivor"))).to(CharacterType::Survivor),
            choice((phrase("spirit animals"), phrase("spirit animal")))
                .to(CharacterType::SpiritAnimal),
        )))
        .then_ignore(phrase("}"))
        .boxed()
}

fn fast_card<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    phrase("'$fast'")
        .ignore_then(non_recursive_predicate())
        .map(|target| CardPredicate::Fast { target: Box::new(target) })
        .boxed()
}

fn character<'a>() -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    choice((phrase("characters"), phrase("character"))).boxed()
}

fn character_with_spark_compared_to_abandoned_this_turn<'a>(
) -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then_ignore(phrase("with spark x"))
        .then(choice((
            phrase("or less").to(Operator::OrLess),
            phrase("or more").to(Operator::OrMore),
        )))
        .then_ignore(phrase(", where x is the number of characters you have abandoned this turn"))
        .map(|(target, spark_operator)| {
            CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn {
                target: Box::new(target),
                spark_operator,
            }
        })
        .boxed()
}

fn character_with_spark_compared_to_abandoned<'a>(
) -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then_ignore(phrase("with spark"))
        .then(choice((
            phrase("less than or equal to").to(Operator::OrLess),
            phrase("equal to").to(Operator::Exactly),
            phrase("greater than or equal to").to(Operator::OrMore),
        )))
        .then_ignore(phrase("the abandoned character's spark"))
        .map(|(target, spark_operator)| CardPredicate::CharacterWithSparkComparedToAbandoned {
            target: Box::new(target),
            spark_operator,
        })
        .boxed()
}
