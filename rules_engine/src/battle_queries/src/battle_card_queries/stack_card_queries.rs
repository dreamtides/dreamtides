use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::stack_card_state::{EffectTargets, StackItemId};

/// Returns the current valid targets to display for an item on the stack, if
/// any.
pub fn displayed_targets(
    battle: &BattleState,
    item: impl Into<StackItemId>,
) -> Option<&EffectTargets> {
    let stack_card = battle.cards.stack_item(item)?;
    validate_targets(battle, stack_card.targets.as_ref())
}

/// Returns the provided [EffectTargets] if they are all valid, or otherwise
/// None.
pub fn validate_targets<'a>(
    battle: &BattleState,
    targets: Option<&'a EffectTargets>,
) -> Option<&'a EffectTargets> {
    if targets_are_valid(battle, targets) { targets } else { None }
}

fn targets_are_valid(battle: &BattleState, targets: Option<&EffectTargets>) -> bool {
    match targets {
        Some(EffectTargets::Character(character_id, object_id)) => {
            battle.cards.is_valid_object_id(character_id.card_id(), *object_id)
        }
        Some(EffectTargets::StackCard(stack_card_id, object_id)) => {
            battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id)
        }
        None => true,
    }
}
