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
    words(&["with", "cost"])
        .ignore_then(energy_comparison_operator())
        .then_ignore(words(&["the", "number", "of", "allied"]))
        .then(subtype())
        .map(|(operator, count_matching)| (operator, CardPredicate::CharacterType(count_matching)))
}

pub fn with_cost_compared_to_void_count_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Energy>, ParserExtra<'a>> + Clone {
    words(&["with", "cost"])
        .ignore_then(energy_comparison_operator())
        .then_ignore(words(&["the", "number", "of", "cards", "in", "your", "void"]))
}

pub fn with_spark_compared_to_abandoned_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    words(&["with", "spark"])
        .ignore_then(spark_comparison_operator())
        .then_ignore(words(&["that", "ally's", "spark"]))
}

pub fn with_spark_compared_to_energy_spent_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    words(&["with", "spark"])
        .ignore_then(spark_comparison_operator())
        .then_ignore(words(&["the", "amount", "of"]))
        .then_ignore(directive("energy-symbol"))
        .then_ignore(word("paid"))
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

fn spark_comparison_operator<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Spark>, ParserExtra<'a>> + Clone {
    choice((
        words(&["less", "than"]).to(Operator::OrLess),
        words(&["greater", "than"]).to(Operator::OrMore),
        words(&["equal", "to"]).to(Operator::Exactly),
    ))
}

fn energy_comparison_operator<'a>(
) -> impl Parser<'a, ParserInput<'a>, Operator<Energy>, ParserExtra<'a>> + Clone {
    choice((
        words(&["less", "than"]).to(Operator::OrLess),
        words(&["greater", "than"]).to(Operator::OrMore),
        words(&["equal", "to"]).to(Operator::Exactly),
    ))
}
