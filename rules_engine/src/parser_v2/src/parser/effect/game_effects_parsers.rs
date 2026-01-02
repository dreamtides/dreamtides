use ability_data::collection_expression::CollectionExpression;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, directive, foresee_count, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        foresee(),
        discover(),
        counterspell(),
        choice((dissolve_all_characters(), dissolve_character())).boxed(),
        choice((banish_character(), banish_enemy_void())).boxed(),
    ))
    .boxed()
}

pub fn foresee<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    foresee_count().map(|count| StandardEffect::Foresee { count })
}

pub fn discover<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("discover")
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .map(|predicate| StandardEffect::Discover { predicate })
}

pub fn counterspell<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("prevent")
        .ignore_then(article())
        .ignore_then(word("played").or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::Counterspell { target })
}

pub fn dissolve_all_characters<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve").ignore_then(word("all")).ignore_then(word("characters")).map(|_| {
        StandardEffect::DissolveCharactersCount {
            target: Predicate::Any(CardPredicate::Character),
            count: CollectionExpression::All,
        }
    })
}

pub fn dissolve_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::DissolveCharacter { target })
}

pub fn banish_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::BanishCharacter { target })
}

pub fn banish_enemy_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(words(&["the", "opponent's", "void"]))
        .map(|_| StandardEffect::BanishEnemyVoid)
}
