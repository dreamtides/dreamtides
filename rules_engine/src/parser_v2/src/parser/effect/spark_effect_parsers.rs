use ability_data::collection_expression::CollectionExpression;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Spark;

use crate::parser::parser_helpers::{
    article, kindle_amount, spark, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser, quantity_expression_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        kindle(),
        spark_becomes(),
        each_allied_gains_spark(),
        gains_spark_for_each(),
        gains_spark(),
    ))
    .boxed()
}

pub fn kindle<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    kindle_amount().map(|amount| StandardEffect::Kindle { amount: Spark(amount) })
}

fn gains_spark_for_each<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        words(&["gain", "+"])
            .ignore_then(spark())
            .then_ignore(word("spark"))
            .then_ignore(words(&["for", "each"]))
            .then(quantity_expression_parser::parser())
            .map(|(gains, for_quantity)| StandardEffect::GainsSparkForQuantity {
                target: Predicate::This,
                gains: Spark(gains),
                for_quantity,
            }),
        article()
            .or_not()
            .ignore_then(predicate_parser::predicate_parser())
            .then_ignore(words(&["gains", "+"]))
            .then(spark())
            .then_ignore(word("spark"))
            .then_ignore(words(&["for", "each"]))
            .then(quantity_expression_parser::parser())
            .map(|((target, gains), for_quantity)| StandardEffect::GainsSparkForQuantity {
                target,
                gains: Spark(gains),
                for_quantity,
            }),
    ))
    .boxed()
}

fn each_allied_gains_spark<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["have", "each", "allied"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["gain", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .map(|(matching, gains)| StandardEffect::EachMatchingGainsSpark {
            each: matching,
            gains: Spark(gains),
        })
}

fn gains_spark<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    article()
        .or_not()
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["gains", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .map(|(target, gains)| StandardEffect::GainsSpark { target, gains: Spark(gains) })
}

fn spark_becomes<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    words(&["the", "spark", "of", "each", "allied"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(word("becomes"))
        .then(spark())
        .map(|(matching, spark_value)| StandardEffect::SparkBecomes {
            collection: CollectionExpression::All,
            matching,
            spark: Spark(spark_value),
        })
}
