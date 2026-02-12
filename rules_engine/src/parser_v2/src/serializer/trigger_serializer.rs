use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use strings::strings;

use crate::serializer::predicate_serializer;

/// Serializes a trigger event to its template text representation.
pub fn serialize_trigger_event(trigger: &TriggerEvent) -> String {
    match trigger {
        TriggerEvent::Keywords(keywords) => serialize_keyword_trigger(keywords),
        TriggerEvent::Play(predicate) => {
            strings::when_you_play_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::OpponentPlays(predicate) => strings::when_opponent_plays_trigger(
            predicate_serializer::serialize_predicate(predicate),
        )
        .to_string(),
        TriggerEvent::PlayFromHand(predicate) => strings::when_you_play_from_hand_trigger(
            predicate_serializer::serialize_predicate(predicate),
        )
        .to_string(),
        TriggerEvent::PlayCardsInTurn(count) => {
            strings::when_you_play_cards_in_turn_trigger(*count).to_string()
        }
        TriggerEvent::PlayDuringTurn(predicate, turn) => match turn {
            PlayerTurn::YourTurn => strings::when_you_play_in_turn_trigger(
                predicate_serializer::serialize_predicate(predicate),
            )
            .to_string(),
            PlayerTurn::EnemyTurn => strings::when_you_play_during_enemy_turn_trigger(
                predicate_serializer::serialize_predicate(predicate),
            )
            .to_string(),
        },
        TriggerEvent::Discard(predicate) => {
            strings::when_you_discard_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::Materialize(predicate) => strings::when_you_materialize_trigger(
            predicate_serializer::serialize_predicate(predicate),
        )
        .to_string(),
        TriggerEvent::Dissolved(predicate) => {
            strings::when_dissolved_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::Banished(predicate) => {
            strings::when_banished_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::LeavesPlay(predicate) => {
            strings::when_leaves_play_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::Abandon(predicate) => {
            strings::when_you_abandon_trigger(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        TriggerEvent::AbandonCardsInTurn(count) => {
            strings::when_you_abandon_count_in_turn_trigger(*count).to_string()
        }
        TriggerEvent::PutIntoVoid(predicate) => strings::when_put_into_void_trigger(
            predicate_serializer::serialize_predicate(predicate),
        )
        .to_string(),
        TriggerEvent::DrawCardsInTurn(count) => {
            strings::when_you_draw_in_turn_trigger(*count).to_string()
        }
        TriggerEvent::EndOfYourTurn => strings::at_end_of_your_turn_trigger().to_string(),
        TriggerEvent::DrawAllCardsInCopyOfDeck => strings::when_deck_empty_trigger().to_string(),
        TriggerEvent::MaterializeNthThisTurn(predicate, count) => {
            strings::when_you_materialize_nth_in_turn_trigger(
                *count,
                predicate_serializer::serialize_predicate_plural(predicate),
            )
            .to_string()
        }
        TriggerEvent::GainEnergy => strings::when_you_gain_energy_trigger().to_string(),
    }
}

/// Serializes a keyword trigger list to its phrase-driven text representation.
fn serialize_keyword_trigger(keywords: &[TriggerKeyword]) -> String {
    match keywords {
        [TriggerKeyword::Judgment] => strings::judgment().to_string(),
        [TriggerKeyword::Materialized] => strings::materialized().to_string(),
        [TriggerKeyword::Dissolved] => strings::dissolved().to_string(),
        [TriggerKeyword::Materialized, TriggerKeyword::Judgment] => {
            strings::materialized_judgment().to_string()
        }
        [TriggerKeyword::Materialized, TriggerKeyword::Dissolved] => {
            strings::materialized_dissolved().to_string()
        }
        _ => {
            let keyword_text =
                keywords.iter().map(|k| serialize_keyword_name(k)).collect::<Vec<_>>().join(", ");
            strings::trigger(keyword_text).to_string()
        }
    }
}

/// Returns the display name for a trigger keyword.
fn serialize_keyword_name(keyword: &TriggerKeyword) -> &'static str {
    match keyword {
        TriggerKeyword::Judgment => "Judgment",
        TriggerKeyword::Materialized => "Materialized",
        TriggerKeyword::Dissolved => "Dissolved",
    }
}
