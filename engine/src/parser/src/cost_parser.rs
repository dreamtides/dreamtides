use ability_data::cost::Cost;
use ability_data::predicate::Predicate;
use chumsky::prelude::*;
use chumsky::Parser;
use core_data::numerics::Energy;

use crate::parser_utils::{count, numeric, phrase, ErrorType};
use crate::{card_predicate_parser, determiner_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        numeric("$", Energy, "").map(Cost::Energy),
        numeric("pay $", Energy, "").map(Cost::Energy),
        phrase("banish a card from your void").to(Cost::BanishCardsFromYourVoid(1)),
        numeric("banish", count, "cards from your void").map(Cost::BanishCardsFromYourVoid),
        phrase("banish a card from the enemy's void").to(Cost::BanishCardsFromEnemyVoid(1)),
        numeric("banish", count, "cards from the enemy's void").map(Cost::BanishCardsFromEnemyVoid),
        phrase("abandon")
            .ignore_then(determiner_parser::your_action())
            .map(|p| Cost::AbandonCharacters(p, 1)),
        phrase("discard your hand").to(Cost::DiscardHand),
    ))
    .boxed()
}

/// Alternate phrasing for costs, which are written in static abilities, for
/// example "You may play this event for $0 by abandoning a character".
pub fn inflected_additional_cost<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("banishing another card from your void").to(Cost::BanishCardsFromYourVoid(1)),
        phrase("banishing all other cards from your void").to(Cost::BanishAllCardsFromYourVoid),
        phrase("banishing all cards from your void").to(Cost::BanishAllCardsFromYourVoid),
        choice((phrase("abandoning a").to(1), numeric("abandoning", count, "")))
            .then(card_predicate_parser::parser())
            .map(|(n, predicate)| Cost::AbandonCharacters(Predicate::Your(predicate), n)),
        phrase("banishing")
            .ignore_then(determiner_parser::your_action())
            .then_ignore(phrase("from your hand"))
            .map(Cost::BanishFromHand),
    ))
    .boxed()
}
