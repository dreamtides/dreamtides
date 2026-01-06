use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};

use super::predicate_serializer::serialize_predicate;

pub fn serialize_trigger_event(trigger: &TriggerEvent) -> String {
    match trigger {
        TriggerEvent::Keywords(keywords) if keywords.len() == 1 => {
            format!("{{{}}}", serialize_keyword(&keywords[0]))
        }
        TriggerEvent::Keywords(keywords) if keywords.len() == 2 => {
            format!("{{{}{}}}", serialize_keyword(&keywords[0]), serialize_keyword(&keywords[1]))
        }
        TriggerEvent::Play(predicate) => {
            format!("when you play {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::OpponentPlays(predicate) => {
            format!("when the opponent plays {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::PlayFromHand(predicate) => {
            format!("when you play {} from your hand, ", serialize_predicate(predicate))
        }
        TriggerEvent::PlayCardsInTurn(_) => "when you play {cards-numeral} in a turn, ".to_string(),
        TriggerEvent::PlayDuringTurn(predicate, turn) => match turn {
            PlayerTurn::YourTurn => {
                format!("when you play {} in a turn, ", serialize_predicate(predicate))
            }
            PlayerTurn::EnemyTurn => {
                format!(
                    "when you play {} during the opponent's turn, ",
                    serialize_predicate(predicate)
                )
            }
        },
        TriggerEvent::Discard(predicate) => {
            format!("when you discard {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Materialize(predicate) => {
            format!("when you {{materialize}} {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Dissolved(predicate) => {
            format!("when {} is {{dissolved}}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Banished(predicate) => {
            format!("when {} is {{banished}}, ", serialize_predicate(predicate))
        }
        TriggerEvent::LeavesPlay(predicate) => {
            format!("when {} leaves play, ", serialize_predicate(predicate))
        }
        TriggerEvent::Abandon(predicate) => {
            format!("when you abandon {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::AbandonCardsInTurn(_) => {
            "when you abandon {count-allies} in a turn, ".to_string()
        }
        TriggerEvent::PutIntoVoid(predicate) => {
            format!("when {} is put into your void, ", serialize_predicate(predicate))
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
