use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use ability_data::variable_value::VariableValue;
use strings::strings;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;

/// Serializes a trigger event to its template text representation.
pub fn serialize_trigger_event(trigger: &TriggerEvent, bindings: &mut VariableBindings) -> String {
    match trigger {
        // Keyword arms use Rust format strings because the keyword text is
        // dynamic (produced by serialize_keyword), not a static phrase.
        TriggerEvent::Keywords(keywords) if keywords.len() == 1 => {
            format!("{{{}}}", serialize_keyword(&keywords[0]))
        }
        TriggerEvent::Keywords(keywords) if keywords.len() == 2 => {
            format!("{{{}_{}}}", serialize_keyword(&keywords[0]), serialize_keyword(&keywords[1]))
        }
        TriggerEvent::Play(predicate) => strings::when_you_play_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::OpponentPlays(predicate) => strings::when_opponent_plays_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::PlayFromHand(predicate) => strings::when_you_play_from_hand_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::PlayCardsInTurn(count) => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::when_you_play_cards_in_turn_trigger(0).to_string()
        }
        TriggerEvent::PlayDuringTurn(predicate, turn) => match turn {
            PlayerTurn::YourTurn => strings::when_you_play_in_turn_trigger(
                predicate_serializer::serialize_predicate(predicate, bindings),
            )
            .to_string(),
            PlayerTurn::EnemyTurn => strings::when_you_play_during_enemy_turn_trigger(
                predicate_serializer::serialize_predicate(predicate, bindings),
            )
            .to_string(),
        },
        TriggerEvent::Discard(predicate) => strings::when_you_discard_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::Materialize(predicate) => strings::when_you_materialize_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::Dissolved(predicate) => strings::when_dissolved_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::Banished(predicate) => strings::when_banished_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::LeavesPlay(predicate) => strings::when_leaves_play_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::Abandon(predicate) => strings::when_you_abandon_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::AbandonCardsInTurn(count) => {
            bindings.insert("a".to_string(), VariableValue::Integer(*count));
            strings::when_you_abandon_count_in_turn_trigger(0).to_string()
        }
        TriggerEvent::PutIntoVoid(predicate) => strings::when_put_into_void_trigger(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        TriggerEvent::DrawCardsInTurn(count) => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::when_you_draw_in_turn_trigger(0).to_string()
        }
        TriggerEvent::EndOfYourTurn => strings::at_end_of_your_turn_trigger().to_string(),
        TriggerEvent::DrawAllCardsInCopyOfDeck => strings::when_deck_empty_trigger().to_string(),
        TriggerEvent::MaterializeNthThisTurn(predicate, count) => {
            bindings.insert("n".to_string(), VariableValue::Integer(*count));
            strings::when_you_materialize_nth_in_turn_trigger(
                0,
                predicate_serializer::serialize_predicate_plural(predicate, bindings),
            )
            .to_string()
        }
        TriggerEvent::GainEnergy => strings::when_you_gain_energy_trigger().to_string(),
        // Keyword fallback arm uses Rust format strings because the keyword
        // text is dynamic (produced by serialize_keyword), not a static phrase.
        TriggerEvent::Keywords(keywords) => {
            let keyword_text = keywords.iter().map(serialize_keyword).collect::<Vec<_>>().join("_");
            format!("{{{}}}", keyword_text)
        }
    }
}

/// Serializes a trigger keyword to its plain text name.
pub fn serialize_keyword(keyword: &TriggerKeyword) -> String {
    match keyword {
        TriggerKeyword::Judgment => "Judgment".to_string(),
        TriggerKeyword::Materialized => "Materialized".to_string(),
        TriggerKeyword::Dissolved => "Dissolved".to_string(),
    }
}
