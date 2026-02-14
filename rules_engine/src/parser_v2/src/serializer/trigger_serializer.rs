use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use rlf::Phrase;
use strings::strings;

use crate::serializer::predicate_serializer;

/// Serializes a trigger event to its phrase representation.
pub fn serialize_trigger_event(trigger: &TriggerEvent) -> Phrase {
    match trigger {
        TriggerEvent::Keywords(keywords) => serialize_keyword_trigger(keywords),
        TriggerEvent::Play(predicate) => {
            strings::when_you_play_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::OpponentPlays(predicate) => strings::when_opponent_plays_trigger(
            predicate_serializer::serialize_predicate(predicate),
        ),
        TriggerEvent::PlayFromHand(predicate) => strings::when_you_play_from_hand_trigger(
            predicate_serializer::serialize_predicate(predicate),
        ),
        TriggerEvent::PlayCardsInTurn(count) => {
            strings::when_you_play_cards_in_turn_trigger(*count)
        }
        TriggerEvent::PlayDuringTurn(predicate, turn) => match turn {
            PlayerTurn::YourTurn => strings::when_you_play_in_turn_trigger(
                predicate_serializer::serialize_predicate(predicate),
            ),
            PlayerTurn::EnemyTurn => strings::when_you_play_during_enemy_turn_trigger(
                predicate_serializer::serialize_predicate(predicate),
            ),
        },
        TriggerEvent::Discard(predicate) => {
            strings::when_you_discard_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::Materialize(predicate) => strings::when_you_materialize_trigger(
            predicate_serializer::serialize_predicate(predicate),
        ),
        TriggerEvent::Dissolved(predicate) => {
            strings::when_dissolved_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::Banished(predicate) => {
            strings::when_banished_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::LeavesPlay(predicate) => {
            strings::when_leaves_play_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::Abandon(predicate) => {
            strings::when_you_abandon_trigger(predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::AbandonCardsInTurn(count) => {
            strings::when_you_abandon_count_in_turn_trigger(*count)
        }
        TriggerEvent::PutIntoVoid(predicate) => strings::when_put_into_void_trigger(
            predicate_serializer::serialize_predicate(predicate),
        ),
        TriggerEvent::DrawCardsInTurn(count) => strings::when_you_draw_in_turn_trigger(*count),
        TriggerEvent::EndOfYourTurn => strings::at_end_of_your_turn_trigger(),
        TriggerEvent::DrawAllCardsInCopyOfDeck => strings::when_deck_empty_trigger(),
        TriggerEvent::MaterializeNthThisTurn(predicate, count) => {
            strings::when_you_materialize_nth_in_turn_trigger(
                *count,
                predicate_serializer::serialize_predicate(predicate),
            )
        }
        TriggerEvent::GainEnergy => strings::when_you_gain_energy_trigger(),
    }
}

/// Serializes a keyword trigger list to its phrase representation.
fn serialize_keyword_trigger(keywords: &[TriggerKeyword]) -> Phrase {
    match keywords {
        [TriggerKeyword::Judgment] => strings::judgment(),
        [TriggerKeyword::Materialized] => strings::materialized(),
        [TriggerKeyword::Dissolved] => strings::dissolved(),
        [TriggerKeyword::Materialized, TriggerKeyword::Judgment] => {
            strings::materialized_judgment()
        }
        [TriggerKeyword::Materialized, TriggerKeyword::Dissolved] => {
            strings::materialized_dissolved()
        }
        _ => {
            let keyword_text =
                keywords.iter().map(serialize_keyword_name).collect::<Vec<_>>().join(", ");
            strings::trigger(keyword_text)
        }
    }
}

/// Returns the display name for a trigger keyword.
fn serialize_keyword_name(keyword: &TriggerKeyword) -> String {
    match keyword {
        TriggerKeyword::Judgment => strings::judgment_keyword_name().to_string(),
        TriggerKeyword::Materialized => strings::materialized_keyword_name().to_string(),
        TriggerKeyword::Dissolved => strings::dissolved_keyword_name().to_string(),
    }
}
