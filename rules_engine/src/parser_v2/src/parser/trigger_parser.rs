use ability_data::predicate::Predicate;
use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, cards_numeral, comma, directive, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn trigger_event_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    choice((keyword_triggers(), standard_trigger().then_ignore(comma()))).boxed()
}

fn keyword_triggers<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    choice((keyword_trigger(), combined_keyword_trigger()))
}

fn keyword_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    choice((
        directive("judgment").to(TriggerKeyword::Judgment),
        directive("materialized").to(TriggerKeyword::Materialized),
        directive("dissolved").to(TriggerKeyword::Dissolved),
    ))
    .map(|keyword| TriggerEvent::Keywords(vec![keyword]))
}

fn combined_keyword_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    choice((
        directive("materializedjudgment")
            .to(vec![TriggerKeyword::Materialized, TriggerKeyword::Judgment]),
        directive("materializeddissolved")
            .to(vec![TriggerKeyword::Materialized, TriggerKeyword::Dissolved]),
    ))
    .map(TriggerEvent::Keywords)
}

fn standard_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    choice((play_triggers(), action_triggers(), state_change_triggers(), timing_triggers())).boxed()
}

fn play_triggers<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    choice((
        play_cards_in_turn_trigger(),
        play_from_hand_trigger(),
        play_during_turn_trigger(),
        play_trigger(),
    ))
    .boxed()
}

fn action_triggers<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    choice((discard_trigger(), materialize_trigger(), abandon_trigger())).boxed()
}

fn state_change_triggers<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    choice((
        dissolved_trigger(),
        banished_trigger(),
        leaves_play_trigger(),
        put_into_void_trigger(),
    ))
    .boxed()
}

fn timing_triggers<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    choice((draw_all_cards_trigger(), end_of_turn_trigger(), gain_energy_trigger())).boxed()
}

fn draw_all_cards_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "have", "no", "cards", "in", "your", "deck"])
        .to(TriggerEvent::DrawAllCardsInCopyOfDeck)
}

fn play_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .map(|card_predicate| TriggerEvent::Play(Predicate::Your(card_predicate)))
}

fn play_from_hand_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "your", "hand"]))
        .map(|card_predicate| TriggerEvent::PlayFromHand(Predicate::Your(card_predicate)))
}

fn play_during_turn_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["in", "a", "turn"]))
        .map(|card_predicate| {
            TriggerEvent::PlayDuringTurn(Predicate::Your(card_predicate), PlayerTurn::YourTurn)
        })
}

fn play_cards_in_turn_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(cards_numeral())
        .then_ignore(words(&["in", "a", "turn"]))
        .map(TriggerEvent::PlayCardsInTurn)
}

fn discard_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when", "you", "discard"])
        .ignore_then(article().or_not())
        .ignore_then(choice((
            card_predicate_parser::parser().map(Predicate::Your),
            predicate_parser::predicate_parser(),
        )))
        .map(TriggerEvent::Discard)
}

fn materialize_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you"])
        .ignore_then(directive("materialize"))
        .ignore_then(article().or_not())
        .ignore_then(choice((
            card_predicate_parser::parser().map(Predicate::Your),
            predicate_parser::predicate_parser(),
        )))
        .map(TriggerEvent::Materialize)
}

fn dissolved_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(word("is"))
        .then_ignore(directive("dissolved"))
        .map(TriggerEvent::Dissolved)
}

fn banished_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(word("is"))
        .then_ignore(directive("banished"))
        .map(TriggerEvent::Banished)
}

fn leaves_play_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["leaves", "play"]))
        .map(TriggerEvent::LeavesPlay)
}

fn put_into_void_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["is", "put", "into", "your", "void"]))
        .map(TriggerEvent::PutIntoVoid)
}

fn abandon_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when", "you", "abandon"])
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(TriggerEvent::Abandon)
}

fn end_of_turn_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["at", "the", "end", "of", "your", "turn"]).to(TriggerEvent::EndOfYourTurn)
}

fn gain_energy_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "gain", "energy"]).to(TriggerEvent::GainEnergy)
}
