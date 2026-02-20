use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;
use core_data::card_types::CardSubtype;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{
    comma, count, count_allied_subtype, count_allies, subtype, word, words, ParserExtra,
    ParserInput,
};

pub fn condition_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone
{
    choice((
        while_you_have_count_or_more_cards_in_your_void(),
        this_card_is_in_your_void(),
        dissolved_this_turn(),
        discarded_this_turn(),
        with_count_allies_that_share_a_character_type(),
        with_count_allied_subtype(),
        with_an_allied_subtype(),
    ))
    .boxed()
}

fn with_count_allied_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    word("with").ignore_then(count_allied_subtype()).then_ignore(comma()).map(|(count, subtype)| {
        Condition::PredicateCount { count, predicate: allied_subtype_predicate(subtype) }
    })
}

fn with_an_allied_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    words(&["with", "an", "allied"]).ignore_then(subtype()).then_ignore(comma()).map(|subtype| {
        Condition::PredicateCount { count: 1, predicate: allied_subtype_predicate(subtype) }
    })
}

fn with_count_allies_that_share_a_character_type<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    word("with")
        .ignore_then(count_allies())
        .then_ignore(words(&["that", "share", "a", "character", "type"]))
        .then_ignore(comma())
        .map(|count| Condition::AlliesThatShareACharacterType { count })
}

fn allied_subtype_predicate(subtype: CardSubtype) -> Predicate {
    Predicate::Another(CardPredicate::CharacterType(subtype))
}

fn dissolved_this_turn<'a>() -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone
{
    word("a")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["dissolved", "this", "turn"]))
        .map(|predicate| Condition::DissolvedThisTurn { predicate: Predicate::Any(predicate) })
}

fn discarded_this_turn<'a>() -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone
{
    words(&["you", "have", "discarded"])
        .ignore_then(word("a").or_not())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["this", "turn"]))
        .map(|predicate| Condition::CardsDiscardedThisTurn { count: 1, predicate })
}

fn this_card_is_in_your_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    word("if")
        .ignore_then(words(&["this", "card", "is", "in", "your", "void"]))
        .then_ignore(comma())
        .to(Condition::ThisCardIsInYourVoid)
}

fn while_you_have_count_or_more_cards_in_your_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    words(&["while", "you", "have"])
        .ignore_then(count())
        .then_ignore(words(&["or", "more", "cards", "in", "your", "void"]))
        .then_ignore(comma())
        .map(|count| Condition::CardsInVoidCount { count })
}
