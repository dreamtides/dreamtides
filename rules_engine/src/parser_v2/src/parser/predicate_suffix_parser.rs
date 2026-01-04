use ability_data::predicate::Operator;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    directive, energy, spark, word, words, ParserExtra, ParserInput,
};

pub fn with_cost_suffix<'a>(
) -> impl Parser<'a, ParserInput<'a>, (u32, Operator<Energy>), ParserExtra<'a>> + Clone {
    words(&["with", "cost"])
        .ignore_then(energy())
        .then(energy_operator().or_not())
        .map(|(cost_value, operator)| (cost_value, operator.unwrap_or(Operator::Exactly)))
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
    ))
}
