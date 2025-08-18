use ability_data::predicate::{CardPredicate, Operator};
use chumsky::Parser;
use chumsky::prelude::choice;
use core_data::card_types::CardSubtype;
use core_data::numerics::{Energy, Spark};

use crate::parser_utils::{ErrorType, a_or_an, numeric, phrase};

pub fn parser<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    choice((
        character_with_cost_compared_to_controlled(),
        character_with_cost_compared_to_abandoned(),
        character_with_spark_compared_to_abandoned_this_turn(),
        character_with_spark_compared_to_abandoned(),
        fast_card(),
        card_with_cost(),
        non_recursive_predicate(),
    ))
}

fn non_recursive_predicate<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    choice((
        character_with_spark(),
        character_with_materialized_ability(),
        character_with_multi_activated_ability(),
        character_type().map(CardPredicate::CharacterType),
        phrase("character that is not")
            .ignore_then(a_or_an())
            .ignore_then(character_type())
            .map(CardPredicate::NotCharacterType),
        phrase("characters that are not")
            .ignore_then(character_type())
            .map(CardPredicate::NotCharacterType),
        choice((phrase("cards"), phrase("card"))).to(CardPredicate::Card),
        character().to(CardPredicate::Character),
        choice((phrase("events"), phrase("event"))).to(CardPredicate::Event),
    ))
    .boxed()
}

fn card_with_cost<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    non_recursive_predicate()
        .then(numeric("with cost $", Energy, ""))
        .then(
            choice((
                phrase("or less").to(Operator::OrLess),
                phrase("or more").to(Operator::OrMore),
            ))
            .or_not(),
        )
        .map(|((target, cost), operator)| CardPredicate::CardWithCost {
            target: Box::new(target),
            cost_operator: operator.unwrap_or(Operator::Exactly),
            cost,
        })
}

fn character_with_spark<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(numeric("with spark", Spark, ""))
        .then(choice((
            phrase("or less").to(Operator::OrLess),
            phrase("or more").to(Operator::OrMore),
        )))
        .map(|(spark, operator)| CardPredicate::CharacterWithSpark(spark, operator))
}

fn character_with_cost_compared_to_controlled<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
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
}

fn character_with_cost_compared_to_abandoned<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
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
}

fn character_with_materialized_ability<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(phrase("with a $materialized ability"))
        .to(CardPredicate::CharacterWithMaterializedAbility)
}

fn character_with_multi_activated_ability<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    character()
        .ignore_then(phrase("with a $multiactivated ability"))
        .to(CardPredicate::CharacterWithMultiActivatedAbility)
}

fn character_type<'a>() -> impl Parser<'a, &'a str, CardSubtype, ErrorType<'a>> {
    phrase("{cardtype: ")
        .ignore_then(choice((
            choice((phrase("warriors"), phrase("warrior"))).to(CardSubtype::Warrior),
            choice((phrase("survivors"), phrase("survivor"))).to(CardSubtype::Survivor),
            choice((phrase("spirit animals"), phrase("spirit animal")))
                .to(CardSubtype::SpiritAnimal),
        )))
        .then_ignore(phrase("}"))
}

fn fast_card<'a>() -> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
    phrase("'$fast'")
        .ignore_then(non_recursive_predicate())
        .map(|target| CardPredicate::Fast { target: Box::new(target) })
}

fn character<'a>() -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    choice((phrase("characters"), phrase("character")))
}

fn character_with_spark_compared_to_abandoned_this_turn<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
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
}

fn character_with_spark_compared_to_abandoned<'a>()
-> impl Parser<'a, &'a str, CardPredicate, ErrorType<'a>> {
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
}
