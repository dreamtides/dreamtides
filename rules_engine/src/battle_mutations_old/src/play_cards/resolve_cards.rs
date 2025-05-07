use ability_data::ability::Ability;
use assert_with::{assert_that, panic_with};
use battle_data_old::battle::battle_turn_step::BattleTurnStep;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_cards::card_id::StackCardId;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;
use core_data::types::PlayerName;
use logging::battle_trace;

use crate::effects::apply_effect;
use crate::play_cards::character_limit;
use crate::zone_mutations::move_card;

/// Marks a player as having taken the "pass" action on the current stack.
///
/// Card resolution works as follows in Dreamtides: A player may play a card
/// (during their turn or at the end of their opponent's turn), which creates a
/// "stack", which is resolved via a system of "priority":
///
/// - Whenever a card is played, the opponent of the card's controller receives
///   priority.
/// - Whenever a card resolves, the controller of that card receives priority if
///   the stack is not empty.
///
/// Priority cannot be "held", and players cannot add more than one card to the
/// stack at a time.
///
/// When a player has priority, they may either play a card or take the "pass"
/// action. If the player with priority takes the "pass" action, the top card of
/// the stack resolves. Note that only *one* player is ever required to pass
/// priority to resolve a card on the stack.
///
/// Cards resolve in Last In, First Out order, meaning the top card of the stack
/// is resolved first.
pub fn pass_priority(battle: &mut BattleData, player: PlayerName) {
    assert_that!(battle.priority == player, battle, || "Player does not have priority");

    if let Some(card_id) = battle.cards.stack().last() {
        let card_id = *card_id;
        let Some(card) = battle.cards.card(card_id) else {
            panic_with!(battle, "Card not found on stack");
        };
        let controller = card.controller();
        resolve_card(battle, card_id, controller);

        // After a card resolves, its controller receives priority (if stack is not
        // empty)
        if battle.cards.stack().is_empty() {
            battle.priority = if battle.step == BattleTurnStep::Ending {
                battle.turn.active_player.opponent()
            } else {
                battle.turn.active_player
            };
        } else {
            battle.priority = controller;
        }
    } else {
        panic_with!(battle, "No cards on stack");
    }
}

/// Resolves a card currently on the stack, applying its effects and moving it
/// to the appropriate zone.
fn resolve_card(
    battle: &mut BattleData,
    card_id: StackCardId,
    controller: PlayerName,
) -> Option<()> {
    battle_trace!("Resolving card", battle, card_id);
    if battle.cards.card(card_id)?.properties.card_type == CardType::Event {
        apply_event_effects(battle, card_id);
    }

    battle.cards.card_mut(card_id)?.targets.clear();
    battle.cards.card_mut(card_id)?.additional_cost_choices.clear();

    let source = EffectSource::Game { controller: battle.turn.active_player };
    match battle.cards.card(card_id)?.properties.card_type {
        CardType::Character(_) => {
            let character_id = move_card::to_battlefield(battle, source, card_id)?;
            character_limit::apply(battle, source, controller, character_id);
        }
        _ => {
            move_card::to_void(battle, source, card_id);
        }
    }

    Some(())
}

fn apply_event_effects(battle: &mut BattleData, card_id: StackCardId) -> Option<()> {
    let card = battle.cards.card(card_id)?;
    let controller = card.controller();
    let effects = card
        .abilities
        .iter()
        .filter_map(|ability| match ability {
            Ability::Event(effect) => Some(effect),
            _ => None,
        })
        .cloned()
        .collect::<Vec<_>>();

    for (i, event) in effects.into_iter().enumerate() {
        let event_source = EffectSource::Event {
            controller,
            stack_card_id: card_id,
            ability_number: AbilityNumber(i),
        };
        apply_effect::apply(
            battle,
            event_source,
            event.effect,
            battle.cards.card(card_id)?.targets.clone(),
        );
    }
    Some(())
}
