use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, SingleEffectTarget, StackItemId,
};

/// Gets the targets for a card, if they are valid
pub fn targets(battle: &BattleState, stack_item_id: StackItemId) -> Option<&EffectTargets> {
    let item = battle.cards.stack_item(stack_item_id)?;
    valid_targets(battle, item.targets.as_ref())
}

/// Returns the current valid targets to display for an item on the stack, if
/// any.
pub fn displayed_targets(
    battle: &BattleState,
    item: impl Into<StackItemId>,
) -> Option<&EffectTargets> {
    let stack_item = battle.cards.stack_item(item)?;
    valid_targets(battle, stack_item.targets.as_ref())
}

/// Returns the provided [EffectTargets] if they are all valid, or otherwise
/// returns None.
pub fn valid_targets<'a>(
    battle: &BattleState,
    targets: Option<&'a EffectTargets>,
) -> Option<&'a EffectTargets> {
    if targets_are_valid(battle, targets) { targets } else { None }
}

fn targets_are_valid(battle: &BattleState, targets: Option<&EffectTargets>) -> bool {
    match targets {
        Some(EffectTargets::Single(SingleEffectTarget::Character(character_id, object_id))) => {
            battle.cards.is_valid_object_id(character_id.card_id(), *object_id)
        }
        Some(EffectTargets::Single(SingleEffectTarget::StackCard(stack_card_id, object_id))) => {
            battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id)
        }
        Some(EffectTargets::List(targets)) => {
            targets.iter().all(|target| targets_are_valid_single(battle, target))
        }
        None => true,
    }
}

fn targets_are_valid_single(battle: &BattleState, target: &SingleEffectTarget) -> bool {
    match target {
        SingleEffectTarget::Character(character_id, object_id) => {
            battle.cards.is_valid_object_id(character_id.card_id(), *object_id)
        }
        SingleEffectTarget::StackCard(stack_card_id, object_id) => {
            battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id)
        }
    }
}
