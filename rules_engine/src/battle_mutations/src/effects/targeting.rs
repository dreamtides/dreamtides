use battle_state::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::{EffectTargets, StandardEffectTarget};

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

/// Returns the [CardId] for an effect which can target either a stack card or
/// a character.
pub fn stack_or_character_id(targets: &mut Option<EffectTargets>) -> Option<CardId> {
    match targets.take() {
        Some(EffectTargets::Standard(StandardEffectTarget::StackCard(stack_card_id, ..))) => {
            Some(stack_card_id.card_id())
        }
        Some(EffectTargets::Standard(StandardEffectTarget::Character(character_id, ..))) => {
            Some(character_id.card_id())
        }
        Some(EffectTargets::EffectList(mut target_list)) => {
            if let Some(Some(target)) = target_list.pop_front() {
                if !target_list.is_empty() {
                    *targets = Some(EffectTargets::EffectList(target_list));
                }
                match target {
                    StandardEffectTarget::StackCard(stack_card_id, ..) => {
                        Some(stack_card_id.card_id())
                    }
                    StandardEffectTarget::Character(character_id, ..) => {
                        Some(character_id.card_id())
                    }
                    StandardEffectTarget::VoidCards(void_cards) => {
                        void_cards.iter().next().map(|id| id.card_id())
                    }
                }
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
