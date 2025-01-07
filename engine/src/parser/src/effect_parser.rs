use ability_data::effect::{Effect, GameEffect};
use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::{Energy, Spark};

use crate::parser_utils::{count, numeric, phrase, ErrorType};
use crate::{card_predicate_parser, determiner_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    choice((
        discard_cards(),
        dissolve_character(),
        draw_cards(),
        gain_energy(),
        gain_spark_until_next_main_for_each(),
        gain_spark(),
    ))
}

fn draw_cards<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    phrase("draw")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .map(|count| Effect::Effect(GameEffect::DrawCards(count)))
}

fn gain_spark<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    determiner_parser::parser()
        .then(numeric("gains +", Spark, "spark"))
        .map(|(predicate, spark)| Effect::Effect(GameEffect::GainsSpark(predicate, spark)))
}

fn gain_energy<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    numeric("gain $", Energy, "").map(|energy| Effect::Effect(GameEffect::GainEnergy(energy)))
}

fn gain_spark_until_next_main_for_each<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    determiner_parser::parser()
        .then(numeric("gains +", Spark, "spark until your next main phase for each"))
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("you control"))
        .map(|((target, spark), counted)| {
            Effect::Effect(GameEffect::GainsSparkUntilYourNextMainPhaseForEach(
                target,
                spark,
                Predicate::Your(counted),
            ))
        })
}

fn dissolve_character<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    phrase("dissolve")
        .ignore_then(determiner_parser::parser())
        .map(|predicate| Effect::Effect(GameEffect::DissolveCharacter(predicate)))
}

fn discard_cards<'a>() -> impl Parser<'a, &'a str, Effect, ErrorType<'a>> {
    phrase("discard")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .map(|count| Effect::Effect(GameEffect::DiscardCards(count)))
}
