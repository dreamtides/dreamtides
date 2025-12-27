use ability_data::predicate::{CardPredicate, Operator, Predicate};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    directive, energy, spark, subtype, word, words, ParserExtra, ParserInput,
};

pub fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((
        this_parser(),
        it_parser(),
        them_parser(),
        that_parser(),
        enemy_void_parser(),
        your_void_parser(),
        enemy_parser(),
        another_parser(),
        any_other_parser(),
        your_parser(),
        any_parser(),
    ))
    .boxed()
}

fn this_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["this", "character"]).to(Predicate::This),
        words(&["this", "event"]).to(Predicate::This),
        words(&["this", "card"]).to(Predicate::This),
    ))
}

fn it_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("it").to(Predicate::It)
}

fn them_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("them").to(Predicate::Them)
}

fn that_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["that", "character"]).to(Predicate::That),
        words(&["that", "card"]).to(Predicate::That),
        words(&["that", "ally"]).to(Predicate::That),
    ))
}

fn enemy_void_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    words(&["the", "opponent's", "void"])
        .ignore_then(card_predicate_parser())
        .map(Predicate::EnemyVoid)
}

fn your_void_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    words(&["your", "void"]).ignore_then(card_predicate_parser()).map(Predicate::YourVoid)
}

fn enemy_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        word("an").ignore_then(word("enemy")).ignore_then(card_predicate_parser()),
        word("an").ignore_then(word("enemy")).to(CardPredicate::Character),
    ))
    .map(Predicate::Enemy)
}

fn another_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["another", "card"]).ignore_then(card_predicate_parser()).map(Predicate::Another),
        words(&["another", "character"])
            .ignore_then(card_predicate_parser())
            .map(Predicate::Another),
        words(&["another", "ally"]).ignore_then(card_predicate_parser()).map(Predicate::Another),
        words(&["another", "card"]).to(Predicate::Another(CardPredicate::Card)),
        words(&["another", "character"]).to(Predicate::Another(CardPredicate::Character)),
        words(&["another", "ally"]).to(Predicate::Another(CardPredicate::Character)),
    ))
}

fn any_other_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    words(&["any", "other"]).ignore_then(card_predicate_parser()).map(Predicate::AnyOther)
}

fn your_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["a", "character", "you", "control"])
            .ignore_then(card_predicate_parser())
            .map(Predicate::Your),
        words(&["an", "ally"]).ignore_then(card_predicate_parser()).map(Predicate::Your),
        words(&["a", "character", "you", "control"]).to(Predicate::Your(CardPredicate::Character)),
        words(&["an", "ally"]).to(Predicate::Your(CardPredicate::Character)),
        words(&["an", "allied"])
            .ignore_then(subtype())
            .map(|s| Predicate::Your(CardPredicate::CharacterType(s))),
        words(&["allied"])
            .ignore_then(subtype())
            .map(|s| Predicate::Your(CardPredicate::CharacterType(s))),
    ))
}

fn any_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["a", "chosen"]).ignore_then(card_predicate_parser()).map(Predicate::Any),
        words(&["a", "card"]).ignore_then(card_predicate_parser()).map(Predicate::Any),
        words(&["a", "character"]).ignore_then(card_predicate_parser()).map(Predicate::Any),
        words(&["an", "event"]).ignore_then(card_predicate_parser()).map(Predicate::Any),
        words(&["a", "card"]).to(Predicate::Any(CardPredicate::Card)),
        words(&["a", "character"]).to(Predicate::Any(CardPredicate::Character)),
        words(&["an", "event"]).to(Predicate::Any(CardPredicate::Event)),
        subtype().map(|s| Predicate::Any(CardPredicate::CharacterType(s))),
    ))
}

fn card_predicate_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    recursive(|cp| {
        choice((
            fast_parser(cp.clone()),
            character_with_spark_parser(),
            character_with_cost_parser(cp.clone()),
            character_with_cost_compared_to_controlled(),
            character_with_cost_compared_to_abandoned(),
            character_with_spark_compared_to_abandoned(),
            character_with_spark_compared_to_abandoned_count_this_turn(),
            character_with_materialized_ability(),
            character_with_multi_activated_ability(),
            not_character_type_parser(),
            character_type_parser(),
            event_parser(),
            character_parser(),
            card_parser(),
        ))
        .boxed()
    })
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

fn character_type_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    subtype().map(CardPredicate::CharacterType)
}

fn not_character_type_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    word("non").ignore_then(subtype()).map(CardPredicate::NotCharacterType)
}

fn fast_parser<'a>(
    target: impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    directive("fast")
        .ignore_then(target.or(just::<_, ParserInput, ParserExtra>(&[]).to(CardPredicate::Card)))
        .map(|target| CardPredicate::Fast { target: Box::new(target) })
}

fn character_with_spark_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["character", "with"])
            .ignore_then(directive("spark"))
            .ignore_then(spark())
            .then(spark_operator_parser())
            .map(|(s, op)| CardPredicate::CharacterWithSpark(Spark(s), op)),
        words(&["character", "with", "spark"])
            .ignore_then(spark())
            .then(spark_operator_parser())
            .map(|(s, op)| CardPredicate::CharacterWithSpark(Spark(s), op)),
    ))
}

fn character_with_cost_parser<'a>(
    target: impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["character", "with", "cost"])
            .ignore_then(energy())
            .then(energy_operator_parser())
            .map(|(e, op)| CardPredicate::CardWithCost {
                target: Box::new(CardPredicate::Character),
                cost_operator: op,
                cost: Energy(e),
            }),
        words(&["card", "with", "cost"]).ignore_then(energy()).then(energy_operator_parser()).map(
            |(e, op)| CardPredicate::CardWithCost {
                target: Box::new(CardPredicate::Card),
                cost_operator: op,
                cost: Energy(e),
            },
        ),
        words(&["event", "with", "cost"]).ignore_then(energy()).then(energy_operator_parser()).map(
            |(e, op)| CardPredicate::CardWithCost {
                target: Box::new(CardPredicate::Event),
                cost_operator: op,
                cost: Energy(e),
            },
        ),
        target
            .then_ignore(words(&["with", "cost"]))
            .then(energy())
            .then(energy_operator_parser())
            .map(|((target, e), op)| CardPredicate::CardWithCost {
                target: Box::new(target),
                cost_operator: op,
                cost: Energy(e),
            }),
    ))
}

fn character_with_cost_compared_to_controlled<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["character", "with", "cost"])
        .ignore_then(energy_operator_parser())
        .then_ignore(words(&["the", "number", "of"]))
        .then(choice((
            words(&["allied"]).ignore_then(subtype()).map(CardPredicate::CharacterType),
            words(&["cards", "in", "your", "void"]).to(CardPredicate::Card),
        )))
        .map(|(op, count_matching)| CardPredicate::CharacterWithCostComparedToControlled {
            target: Box::new(CardPredicate::Character),
            cost_operator: op,
            count_matching: Box::new(count_matching),
        })
}

fn character_with_cost_compared_to_abandoned<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["character", "with", "cost"])
        .ignore_then(energy_operator_parser())
        .then_ignore(words(&["that", "ally's", "cost"]))
        .map(|op| CardPredicate::CharacterWithCostComparedToAbandoned {
            target: Box::new(CardPredicate::Character),
            cost_operator: op,
        })
}

fn character_with_spark_compared_to_abandoned<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["character", "with", "spark"])
        .ignore_then(spark_operator_parser())
        .then_ignore(words(&["that", "ally's", "spark"]))
        .map(|op| CardPredicate::CharacterWithSparkComparedToAbandoned {
            target: Box::new(CardPredicate::Character),
            spark_operator: op,
        })
}

fn character_with_spark_compared_to_abandoned_count_this_turn<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["character", "with", "spark"])
        .ignore_then(spark_operator_parser())
        .then_ignore(words(&["the", "number", "of", "allies", "abandoned", "this", "turn"]))
        .map(|op| CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn {
            target: Box::new(CardPredicate::Character),
            spark_operator: op,
        })
}

fn character_with_materialized_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["character", "with", "a"])
            .ignore_then(directive("materialized"))
            .ignore_then(word("ability"))
            .to(CardPredicate::CharacterWithMaterializedAbility),
        words(&["character", "with", "an"])
            .ignore_then(directive("materialized"))
            .ignore_then(word("ability"))
            .to(CardPredicate::CharacterWithMaterializedAbility),
    ))
}

fn character_with_multi_activated_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["character", "with", "an", "activated", "ability"])
        .to(CardPredicate::CharacterWithMultiActivatedAbility)
}

fn energy_operator_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Energy>, ParserExtra<'a>> + Clone {
    choice((
        words(&["or", "less"]).to(Operator::OrLess),
        words(&["or", "more"]).to(Operator::OrMore),
        words(&["less", "than"]).to(Operator::LowerBy(Energy(1))),
        word("higher").ignore_then(energy()).map(|e| Operator::HigherBy(Energy(e))),
        word("lower").ignore_then(energy()).map(|e| Operator::LowerBy(Energy(e))),
        empty().to(Operator::Exactly),
    ))
}

fn spark_operator_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    choice((
        words(&["or", "less"]).to(Operator::OrLess),
        words(&["or", "more"]).to(Operator::OrMore),
        words(&["less", "than"]).to(Operator::LowerBy(Spark(1))),
        word("higher").ignore_then(spark()).map(|s| Operator::HigherBy(Spark(s))),
        word("lower").ignore_then(spark()).map(|s| Operator::LowerBy(Spark(s))),
        empty().to(Operator::Exactly),
    ))
}
