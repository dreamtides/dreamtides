use ability_data::trigger_event::{PlayerTurn, TriggerEvent, TriggerKeyword};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::card_ability_queries::trigger_predicates;

/// Returns true if the predicates in a [TriggerEvent] match for the given
/// [Trigger] and thus the trigger should fire.
///
/// # Arguments
///
/// * `battle` - The current battle state.
/// * `trigger` - The trigger to check.
/// * `event` - The event to check.
/// * `owning_card_controller` - The controller of the card that owns the
///   trigger ability we are checking.
/// * `owning_card_id` - The card ID of the card that owns the trigger ability
///   we are checking.
pub fn matches(
    battle: &BattleState,
    trigger: Trigger,
    event: &TriggerEvent,
    owning_card_controller: PlayerName,
    owning_card_id: CardId,
) -> bool {
    match event {
        TriggerEvent::Abandon(predicate) => match trigger {
            Trigger::Abandonded(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::Banished(predicate) => match trigger {
            Trigger::Banished(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::Discard(predicate) => match trigger {
            Trigger::Discarded(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::Dissolved(predicate) => match trigger {
            Trigger::Dissolved(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::PutIntoVoid(predicate) => match trigger {
            Trigger::PutIntoVoid(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::DrawAllCardsInCopyOfDeck => match trigger {
            Trigger::DrewAllCardsInCopyOfDeck(player) => owning_card_controller == player,
            _ => false,
        },
        TriggerEvent::EndOfYourTurn => match trigger {
            Trigger::EndOfTurn(player) => owning_card_controller == player,
            _ => false,
        },
        TriggerEvent::GainEnergy => match trigger {
            Trigger::GainedEnergy(player, _) => owning_card_controller == player,
            _ => false,
        },
        TriggerEvent::Keywords(keywords) => keywords.iter().any(|keyword| {
            matches_keyword(trigger, keyword, owning_card_controller, owning_card_id)
        }),
        TriggerEvent::Materialize(predicate) => match trigger {
            Trigger::Materialized(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::MaterializeNthThisTurn(..) => {
            todo!("Implement MaterializeNthThisTurn")
        }
        TriggerEvent::Play(predicate) => match trigger {
            Trigger::PlayedCard(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
        TriggerEvent::PlayDuringTurn(predicate, player_turn) => match trigger {
            Trigger::PlayedCard(card_id) => {
                let turn_matches = match player_turn {
                    PlayerTurn::YourTurn => battle.turn.active_player == owning_card_controller,
                    PlayerTurn::EnemyTurn => battle.turn.active_player != owning_card_controller,
                };
                turn_matches
                    && trigger_predicates::trigger_matches(
                        battle,
                        predicate,
                        card_id,
                        owning_card_controller,
                        owning_card_id,
                    )
            }
            _ => false,
        },
        TriggerEvent::PlayFromHand(predicate) => match trigger {
            Trigger::PlayedCardFromHand(card_id) => trigger_predicates::trigger_matches(
                battle,
                predicate,
                card_id,
                owning_card_controller,
                owning_card_id,
            ),
            _ => false,
        },
    }
}

/// Returns the card ID of the card that triggered the given [Trigger], if
/// any.
pub fn triggering_card_id(trigger: Trigger) -> Option<CardId> {
    match trigger {
        Trigger::Abandonded(card_id) => Some(card_id.card_id()),
        Trigger::Banished(card_id) => Some(card_id.card_id()),
        Trigger::Discarded(card_id) => Some(card_id.card_id()),
        Trigger::Dissolved(card_id) => Some(card_id.card_id()),
        Trigger::PutIntoVoid(card_id) => Some(card_id.card_id()),
        Trigger::DrewAllCardsInCopyOfDeck(..) => None,
        Trigger::EndOfTurn(..) => None,
        Trigger::GainedEnergy(..) => None,
        Trigger::Judgment(..) => None,
        Trigger::Materialized(card_id) => Some(card_id.card_id()),
        Trigger::PlayedCard(card_id) => Some(card_id.card_id()),
        Trigger::PlayedCardFromHand(card_id) => Some(card_id.card_id()),
        Trigger::PlayedCardFromVoid(card_id) => Some(card_id.card_id()),
    }
}

fn matches_keyword(
    trigger: Trigger,
    keyword: &TriggerKeyword,
    owning_card_controller: PlayerName,
    owning_card_id: CardId,
) -> bool {
    match keyword {
        TriggerKeyword::Materialized => match trigger {
            Trigger::Materialized(card_id) => owning_card_id == card_id.card_id(),
            _ => false,
        },
        TriggerKeyword::Judgment => match trigger {
            Trigger::Judgment(player) => owning_card_controller == player,
            _ => false,
        },
        TriggerKeyword::Dissolved => match trigger {
            Trigger::Dissolved(card_id) => owning_card_id == card_id.card_id(),
            _ => false,
        },
    }
}
