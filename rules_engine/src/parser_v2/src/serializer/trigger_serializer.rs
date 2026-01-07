use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};

use super::predicate_serializer;

pub fn serialize_trigger_event(trigger: &TriggerEvent) -> String {
    match trigger {
        TriggerEvent::Keywords(keywords) if keywords.len() == 1 => {
            format!("{{{}}}", serialize_keyword(&keywords[0]))
        }
        TriggerEvent::Keywords(keywords) if keywords.len() == 2 => {
            format!("{{{}{}}}", serialize_keyword(&keywords[0]), serialize_keyword(&keywords[1]))
        }
        TriggerEvent::Play(predicate) => {
            format!("when you play {}, ", predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::OpponentPlays(predicate) => {
            format!(
                "when the opponent plays {}, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::PlayFromHand(predicate) => {
            format!(
                "when you play {} from your hand, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::PlayCardsInTurn(_) => "when you play {cards-numeral} in a turn, ".to_string(),
        TriggerEvent::PlayDuringTurn(predicate, turn) => match turn {
            PlayerTurn::YourTurn => {
                format!(
                    "when you play {} in a turn, ",
                    predicate_serializer::serialize_predicate(predicate)
                )
            }
            PlayerTurn::EnemyTurn => {
                format!(
                    "when you play {} during the opponent's turn, ",
                    predicate_serializer::serialize_predicate(predicate)
                )
            }
        },
        TriggerEvent::Discard(predicate) => {
            format!("when you discard {}, ", predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::Materialize(predicate) => {
            format!(
                "when you {{materialize}} {}, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::Dissolved(predicate) => {
            format!(
                "when {} is {{dissolved}}, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::Banished(predicate) => {
            format!(
                "when {} is {{banished}}, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::LeavesPlay(predicate) => {
            format!("when {} leaves play, ", predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::Abandon(predicate) => {
            format!("when you abandon {}, ", predicate_serializer::serialize_predicate(predicate))
        }
        TriggerEvent::AbandonCardsInTurn(_) => {
            "when you abandon {count-allies} in a turn, ".to_string()
        }
        TriggerEvent::PutIntoVoid(predicate) => {
            format!(
                "when {} is put into your void, ",
                predicate_serializer::serialize_predicate(predicate)
            )
        }
        TriggerEvent::DrawCardsInTurn(_) => "when you draw {cards-numeral} in a turn, ".to_string(),
        TriggerEvent::EndOfYourTurn => "at the end of your turn, ".to_string(),
        TriggerEvent::DrawAllCardsInCopyOfDeck => {
            "when you have no cards in your deck, ".to_string()
        }
        TriggerEvent::GainEnergy => "when you gain energy, ".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this trigger type"),
    }
}

pub fn serialize_keyword(keyword: &TriggerKeyword) -> String {
    match keyword {
        TriggerKeyword::Judgment => "Judgment".to_string(),
        TriggerKeyword::Materialized => "Materialized".to_string(),
        TriggerKeyword::Dissolved => "Dissolved".to_string(),
    }
}
