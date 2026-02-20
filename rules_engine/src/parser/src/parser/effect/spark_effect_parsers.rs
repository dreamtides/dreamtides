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
        each_gains_spark_for_each(),
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
    article()
        .or_not()
        .ignore_then(predicate_parser::predicate_parser().or_not())
        .then_ignore(choice((word("gains"), word("gain"))))
        .then_ignore(word("+"))
        .then(spark())
        .then_ignore(word("spark"))
        .then_ignore(words(&["for", "each"]))
        .then(quantity_expression_parser::parser())
        .map(|((target, gains), for_quantity)| StandardEffect::GainsSparkForQuantity {
            target: target.unwrap_or(Predicate::This),
            gains: Spark(gains),
            for_quantity,
        })
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

fn each_gains_spark_for_each<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("each")
        .ignore_then(word("allied").or_not())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(word("gains"))
        .then_ignore(word("spark"))
        .then_ignore(words(&["equal", "to", "the", "number", "of"]))
        .then_ignore(word("allied").or_not())
        .then(card_predicate_parser::parser())
        .map(|(each, for_each)| StandardEffect::EachMatchingGainsSparkForEach {
            each,
            gains: Spark(1),
            for_each,
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
