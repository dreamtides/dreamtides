use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;

use crate::card_ability_queries::predicate_matches;

/// Returns true if the predicates in a [TriggerEvent] match for the given
/// [Trigger] and thus the trigger should fire.
pub fn matches(
    battle: &BattleState,
    trigger: Trigger,
    event: &TriggerEvent,
    source: EffectSource,
) -> bool {
    match event {
        TriggerEvent::Abandon(predicate) => match trigger {
            Trigger::Abandonded(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::Banished(predicate) => match trigger {
            Trigger::Banished(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::Discard(predicate) => match trigger {
            Trigger::Discarded(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::Dissolved(predicate) => match trigger {
            Trigger::Dissolved(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::DrawAllCardsInCopyOfDeck => match trigger {
            Trigger::DrewAllCardsInCopyOfDeck(player) => source.controller() == player,
            _ => false,
        },
        TriggerEvent::EndOfYourTurn => match trigger {
            Trigger::EndOfTurn(player) => source.controller() == player,
            _ => false,
        },
        TriggerEvent::GainEnergy => match trigger {
            Trigger::GainedEnergy(player, _) => source.controller() == player,
            _ => false,
        },
        TriggerEvent::Keywords(keywords) => {
            keywords.iter().any(|keyword| matches_keyword(trigger, keyword, source))
        }
        TriggerEvent::Materialize(predicate) => match trigger {
            Trigger::Materialized(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::MaterializeNthThisTurn(..) => {
            todo!("Implement MaterializeNthThisTurn")
        }
        TriggerEvent::Play(predicate) => match trigger {
            Trigger::PlayedCardFromHand(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::PlayDuringTurn(predicate, player_turn) => match trigger {
            Trigger::PlayedCardFromHand(card_id) => {
                let turn_matches = match player_turn {
                    PlayerTurn::YourTurn => battle.turn.active_player == source.controller(),
                    PlayerTurn::EnemyTurn => battle.turn.active_player != source.controller(),
                };
                turn_matches && predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
        TriggerEvent::PlayFromHand(predicate) => match trigger {
            Trigger::PlayedCardFromHand(card_id) => {
                predicate_matches::matches(battle, source, predicate, card_id)
            }
            _ => false,
        },
    }
}

fn matches_keyword(trigger: Trigger, keyword: &TriggerKeyword, source: EffectSource) -> bool {
    match keyword {
        TriggerKeyword::Materialized => match trigger {
            Trigger::Materialized(card_id) => source.card_id() == Some(card_id.card_id()),
            _ => false,
        },
        TriggerKeyword::Judgment => match trigger {
            Trigger::Judgment(player) => source.controller() == player,
            _ => false,
        },
        TriggerKeyword::Dissolved => match trigger {
            Trigger::Dissolved(card_id) => source.card_id() == Some(card_id.card_id()),
            _ => false,
        },
    }
}
