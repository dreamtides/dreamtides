use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, comma, directive, word, words, ParserExtra, ParserInput,
};
use crate::parser::predicate_parser;

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
    choice((
        draw_all_cards_trigger(),
        play_from_hand_trigger(),
        play_during_turn_trigger(),
        play_trigger(),
        discard_trigger(),
        materialize_trigger(),
        dissolved_trigger(),
        banished_trigger(),
        abandon_trigger(),
        end_of_turn_trigger(),
        gain_energy_trigger(),
    ))
}

fn draw_all_cards_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "have", "no", "cards", "in", "your", "deck"])
        .to(TriggerEvent::DrawAllCardsInCopyOfDeck)
}

fn play_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(TriggerEvent::Play)
}

fn play_from_hand_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["from", "your", "hand"]))
        .map(TriggerEvent::PlayFromHand)
}

fn play_during_turn_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you", "play"])
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["in", "a", "turn"]))
        .map(|predicate| TriggerEvent::PlayDuringTurn(predicate, PlayerTurn::YourTurn))
}

fn discard_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when", "you", "discard"])
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(TriggerEvent::Discard)
}

fn materialize_trigger<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone {
    words(&["when", "you"])
        .ignore_then(directive("materialize"))
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
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

fn abandon_trigger<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> + Clone
{
    words(&["when", "you", "abandon"])
        .ignore_then(article())
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
