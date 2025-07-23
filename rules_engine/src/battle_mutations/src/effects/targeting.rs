use std::collections::BTreeSet;

use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId, VoidCardId};
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StandardEffectTarget, VoidCardTarget,
};

/// Returns the [CharacterId] for a set of EffectTargets.
pub fn character_id(targets: &mut Option<EffectTargets>) -> Option<CharacterId> {
    match targets.take() {
        Some(EffectTargets::Standard(StandardEffectTarget::Character(character_id, ..))) => {
            Some(character_id)
        }
        Some(EffectTargets::EffectList(mut target_list)) => {
            if let Some(Some(StandardEffectTarget::Character(character_id, ..))) =
                target_list.pop_front()
            {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                Some(character_id)
            } else {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                None
            }
        }
        _ => None,
    }
}

/// Returns the [StackCardId] for a set of EffectTargets.
pub fn stack_card_id(targets: &mut Option<EffectTargets>) -> Option<StackCardId> {
    match targets.take() {
        Some(EffectTargets::Standard(StandardEffectTarget::StackCard(stack_card_id, ..))) => {
            Some(stack_card_id)
        }
        Some(EffectTargets::EffectList(mut target_list)) => {
            if let Some(Some(StandardEffectTarget::StackCard(stack_card_id, ..))) =
                target_list.pop_front()
            {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                Some(stack_card_id)
            } else {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                None
            }
        }
        _ => None,
    }
}

/// Returns the [VoidCardId] for a set of EffectTargets.
///
/// Panics if there is more than 1 void card in the target set.
/// Returns None if there are no void cards in the target set.
pub fn void_card_id(
    battle: &BattleState,
    targets: &mut Option<EffectTargets>,
) -> Option<VoidCardId> {
    match targets.take() {
        Some(EffectTargets::Standard(StandardEffectTarget::VoidCardSet(void_cards))) => {
            let length = void_cards.len();
            match length {
                0 => None,
                1 => Some(void_cards.into_iter().next().unwrap().id),
                _ => panic_with!("Expected at most 1 void card target", battle, length),
            }
        }
        Some(EffectTargets::EffectList(mut target_list)) => {
            if let Some(Some(StandardEffectTarget::VoidCardSet(void_cards))) =
                target_list.pop_front()
            {
                let length = void_cards.len();
                let result = match length {
                    0 => None,
                    1 => Some(void_cards.into_iter().next().unwrap().id),
                    _ => panic_with!("Expected at most 1 void card target", battle, length),
                };
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                result
            } else {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                None
            }
        }
        _ => None,
    }
}

/// Returns the void card targets for a set of EffectTargets, or None
/// if there are no valid targets.
pub fn void_card_targets(targets: &mut Option<EffectTargets>) -> Option<BTreeSet<VoidCardTarget>> {
    match targets.take() {
        Some(EffectTargets::Standard(StandardEffectTarget::VoidCardSet(void_cards))) => {
            Some(void_cards)
        }
        Some(EffectTargets::EffectList(mut target_list)) => {
            if let Some(Some(StandardEffectTarget::VoidCardSet(void_cards))) =
                target_list.pop_front()
            {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                Some(void_cards)
            } else {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                None
            }
        }
        _ => None,
    }
}
