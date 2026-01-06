use ability_data::predicate::{CardPredicate, Operator, Predicate};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    directive, energy, spark, subtype, word, words, ParserExtra, ParserInput,
};

pub fn with_cost_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (u32, Operator<Energy>), ParserExtra<'a>> + Clone {
    words(&["with", "cost"]).ignore_then(energy()).then(energy_operator().or_not()).map(
        |(cost_value, operator)| match operator {
            Some(op) => match op {
                Operator::HigherBy(_) => (cost_value, Operator::HigherBy(Energy(cost_value))),
                Operator::LowerBy(_) => (cost_value, Operator::LowerBy(Energy(cost_value))),
                _ => (cost_value, op),
            },
            None => (cost_value, Operator::Exactly),
        },
    )
}

pub fn with_spark_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (u32, Operator<Spark>), ParserExtra<'a>> + Clone {
    words(&["with", "spark"])
        .ignore_then(spark())
        .then(spark_operator())
        .map(|(spark_value, operator)| (spark_value, operator))
}

pub fn with_materialized_ability_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    words(&["with", "a"])
        .ignore_then(word("'").or_not())
        .ignore_then(directive("materialized"))
        .ignore_then(word("'").or_not())
        .ignore_then(word("ability"))
        .to(())
}

pub fn with_activated_ability_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    words(&["with", "an", "activated", "ability"]).to(())
}

pub fn which_could_dissolve_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    words(&["event", "which", "could"])
        .ignore_then(directive("dissolve"))
        .ignore_then(words(&["an", "ally"]))
        .to(Predicate::Another(CardPredicate::Character))
}

pub fn with_cost_compared_to_controlled_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (Operator<Energy>, CardPredicate), ParserExtra<'a>> + Clone {
    words(&["with", "cost", "less", "than", "the", "number", "of", "allied"])
        .ignore_then(subtype())
        .map(|count_matching| (Operator::OrLess, CardPredicate::CharacterType(count_matching)))
}

pub fn with_cost_compared_to_void_count_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Energy>, ParserExtra<'a>> + Clone {
    words(&["with", "cost", "less", "than", "the", "number", "of", "cards", "in", "your", "void"])
        .to(Operator::OrLess)
}

pub fn with_spark_compared_to_abandoned_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    words(&["with", "spark", "less", "than", "that", "ally's", "spark"]).to(Operator::OrLess)
}

pub fn with_spark_compared_to_energy_spent_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    words(&["with", "spark", "less", "than", "the", "amount", "of"])
        .ignore_then(directive("energy-symbol"))
        .ignore_then(word("paid"))
        .to(Operator::OrLess)
}

fn spark_operator<'a>() -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone
{
    choice((
        words(&["or", "less"]).to(Operator::OrLess),
        words(&["or", "more"]).to(Operator::OrMore),
    ))
}

fn energy_operator<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Energy>, ParserExtra<'a>> + Clone {
    choice((
        words(&["or", "less"]).to(Operator::OrLess),
        words(&["or", "more"]).to(Operator::OrMore),
        word("higher").to(Operator::HigherBy(Energy(0))),
        word("lower").to(Operator::LowerBy(Energy(0))),
    ))
}
