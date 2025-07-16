use std::collections::VecDeque;

use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StackItemId, StandardEffectTarget,
};

/// Gets the targets for a card, if they are valid
pub fn targets(battle: &BattleState, stack_item_id: StackItemId) -> Option<EffectTargets> {
    let item = battle.cards.stack_item(stack_item_id)?;
    valid_targets(battle, item.targets.as_ref())
}

/// Returns the current valid targets to display for an item on the stack, if
/// any.
pub fn displayed_targets(
    battle: &BattleState,
    item: impl Into<StackItemId>,
) -> Option<EffectTargets> {
    let stack_item = battle.cards.stack_item(item)?;
    valid_targets(battle, stack_item.targets.as_ref())
}

/// Returns valid targets from the requested target set removing e.g. target
/// characters which are no longer in play.
pub fn valid_targets(
    battle: &BattleState,
    targets: Option<&EffectTargets>,
) -> Option<EffectTargets> {
    match targets {
        Some(EffectTargets::Standard(target)) => {
            if is_target_valid(battle, target) {
                Some(targets.unwrap().clone())
            } else {
                None
            }
        }
        Some(EffectTargets::EffectList(target_list)) => {
            let cleaned_targets: VecDeque<Option<StandardEffectTarget>> = target_list
                .iter()
                .map(|target_option| {
                    target_option.as_ref().and_then(|target| {
                        if is_target_valid(battle, target) { Some(target.clone()) } else { None }
                    })
                })
                .collect::<Vec<_>>()
                .into();
            Some(EffectTargets::EffectList(cleaned_targets))
        }
        None => None,
    }
}

fn is_target_valid(battle: &BattleState, target: &StandardEffectTarget) -> bool {
    match target {
        StandardEffectTarget::Character(character_id, object_id) => {
            battle.cards.is_valid_object_id(character_id.card_id(), *object_id)
        }
        StandardEffectTarget::StackCard(stack_card_id, object_id) => {
            battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id)
        }
    }
}
