use ability_data::predicate::{CardPredicate, Operator};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    energy, spark, subtype, word, words, ParserExtra, ParserInput,
};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((with_spark_parser(), with_cost_parser(), subtype_parser(), card_parser())).boxed()
}

fn card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    word("card").to(CardPredicate::Card)
}

fn subtype_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone
{
    subtype().map(CardPredicate::CharacterType)
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

fn with_spark_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    words(&["with", "spark"]).ignore_then(spark()).then(spark_operator()).map(
        |(spark_value, operator)| CardPredicate::CharacterWithSpark(Spark(spark_value), operator),
    )
}

fn with_cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone
{
    words(&["with", "cost"]).ignore_then(energy()).then(energy_operator()).map(
        |(cost_value, operator)| CardPredicate::CardWithCost {
            target: Box::new(CardPredicate::Character),
            cost_operator: operator,
            cost: Energy(cost_value),
        },
    )
}
