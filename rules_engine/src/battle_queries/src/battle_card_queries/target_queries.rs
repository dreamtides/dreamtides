use std::collections::{BTreeSet, VecDeque};

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
            filter_target(battle, target).map(EffectTargets::Standard)
        }
        Some(EffectTargets::EffectList(target_list)) => {
            let cleaned_targets: VecDeque<Option<StandardEffectTarget>> = target_list
                .iter()
                .map(|target_option| {
                    target_option.as_ref().and_then(|target| filter_target(battle, target))
                })
                .collect::<Vec<_>>()
                .into();
            Some(EffectTargets::EffectList(cleaned_targets))
        }
        None => None,
    }
}

fn filter_target(
    battle: &BattleState,
    target: &StandardEffectTarget,
) -> Option<StandardEffectTarget> {
    match target {
        StandardEffectTarget::Character(character_id, object_id) => {
            if battle.cards.is_valid_object_id(character_id.card_id(), *object_id) {
                Some(target.clone())
            } else {
                None
            }
        }
        StandardEffectTarget::StackCard(stack_card_id, object_id) => {
            if battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id) {
                Some(target.clone())
            } else {
                None
            }
        }
        StandardEffectTarget::VoidCards(void_card_set) => {
            let filtered_cards: BTreeSet<_> = void_card_set
                .iter()
                .filter(|void_card_id| {
                    battle.cards.is_valid_object_id(void_card_id.id, void_card_id.object_id)
                })
                .copied()
                .collect();

            if filtered_cards.is_empty() {
                None
            } else {
                Some(StandardEffectTarget::VoidCards(filtered_cards))
            }
        }
    }
}
