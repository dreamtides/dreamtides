use ability_data::predicate::Predicate;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Spark;

use crate::parser::parser_helpers::{kindle_amount, spark, word, words, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((kindle(), gains_spark_for_each(), gains_spark())).boxed()
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
            .then(predicate_parser::predicate_parser())
            .map(|(gains, for_each)| StandardEffect::GainsSparkForQuantity {
                target: Predicate::This,
                gains: Spark(gains),
                for_quantity: QuantityExpression::Matching(for_each),
            }),
        predicate_parser::predicate_parser()
            .then_ignore(words(&["gains", "+"]))
            .then(spark())
            .then_ignore(word("spark"))
            .then_ignore(words(&["for", "each"]))
            .then(predicate_parser::predicate_parser())
            .map(|((target, gains), for_each)| StandardEffect::GainsSparkForQuantity {
                target,
                gains: Spark(gains),
                for_quantity: QuantityExpression::Matching(for_each),
            }),
    ))
    .boxed()
}

fn gains_spark<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    predicate_parser::predicate_parser()
        .then_ignore(words(&["gains", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .map(|(target, gains)| StandardEffect::GainsSpark { target, gains: Spark(gains) })
}
