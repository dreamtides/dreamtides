use ability_data::predicate::Predicate;
use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use chumsky::prelude::choice;
use chumsky::{IterParser, Parser};

use crate::parser_utils::{ErrorType, ordinal_number, phrase};
use crate::{card_predicate_parser, determiner_parser};

pub fn event_parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((
        play_triggers(),
        materialize_triggers(),
        action_triggers(),
        state_triggers(),
        timing_triggers(),
    ))
    .boxed()
}

pub fn keyword_parser<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    let single_keyword = choice((
        phrase("{materialized}").to(TriggerKeyword::Materialized),
        phrase("{judgment}").to(TriggerKeyword::Judgment),
        phrase("{dissolved}").to(TriggerKeyword::Dissolved),
    ));

    single_keyword
        .separated_by(phrase(","))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(TriggerEvent::Keywords)
}

fn materialize<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you materialize")
        .ignore_then(determiner_parser::your_action())
        .map(TriggerEvent::Materialize)
}

fn play_triggers<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((play_from_hand(), play_during_enemy_turn(), play())).boxed()
}

fn materialize_triggers<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((materialize_nth_this_turn(), materialize())).boxed()
}

fn action_triggers<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((discard(), abandon())).boxed()
}

fn state_triggers<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((banished(), dissolved(), put_into_void())).boxed()
}

fn timing_triggers<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((end_of_turn(), gain_energy(), draw_all_cards_in_copy_of_deck())).boxed()
}

fn materialize_nth_this_turn<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you materialize your")
        .ignore_then(ordinal_number())
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("in a turn"))
        .map(|(n, pred)| TriggerEvent::MaterializeNthThisTurn(Predicate::Your(pred), n))
}

fn play_from_hand<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you play")
        .ignore_then(determiner_parser::your_action())
        .then_ignore(phrase("from your hand"))
        .map(TriggerEvent::PlayFromHand)
}

fn play<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you play").ignore_then(determiner_parser::your_action()).map(TriggerEvent::Play)
}

fn play_during_enemy_turn<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you play")
        .ignore_then(determiner_parser::your_action())
        .then_ignore(phrase("during the enemy's turn"))
        .map(|predicate| TriggerEvent::PlayDuringTurn(predicate, PlayerTurn::EnemyTurn))
}

fn discard<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you discard").ignore_then(determiner_parser::your_action()).map(TriggerEvent::Discard)
}

fn end_of_turn<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("the end of your turn").to(TriggerEvent::EndOfYourTurn)
}

fn gain_energy<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you gain energy").to(TriggerEvent::GainEnergy)
}

fn draw_all_cards_in_copy_of_deck<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you draw all of the cards in a copy of your deck")
        .to(TriggerEvent::DrawAllCardsInCopyOfDeck)
}

fn banished<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    choice((
        determiner_parser::target_parser().then_ignore(phrase("is banished")),
        phrase("you banish").ignore_then(determiner_parser::your_action()),
    ))
    .map(TriggerEvent::Banished)
}

fn dissolved<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then_ignore(phrase("is {dissolved}"))
        .map(TriggerEvent::Dissolved)
}

fn put_into_void<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then_ignore(phrase("is put into your void"))
        .map(TriggerEvent::PutIntoVoid)
}

fn abandon<'a>() -> impl Parser<'a, &'a str, TriggerEvent, ErrorType<'a>> {
    phrase("you abandon").ignore_then(determiner_parser::your_action()).map(TriggerEvent::Abandon)
}
