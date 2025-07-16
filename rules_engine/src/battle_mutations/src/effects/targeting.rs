use battle_state::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::EffectTargets;

/// Returns the [CharacterId] for a [EffectTargets::Character] target.
pub fn character_id(targets: Option<&EffectTargets>) -> Option<CharacterId> {
    // ObjectIDs are by validate_targets() before applying effects instead of
    // checking here.
    match targets {
        Some(EffectTargets::Character(character_id, ..)) => Some(*character_id),
        _ => None,
    }
}

/// Returns the [StackCardId] for a [EffectTargets::StackCard] target.
pub fn stack_card_id(targets: Option<&EffectTargets>) -> Option<StackCardId> {
    // ObjectIDs are by validate_targets() before applying effects instead of
    // checking here.
    match targets {
        Some(EffectTargets::StackCard(stack_card_id, ..)) => Some(*stack_card_id),
        _ => None,
    }
}

/// Returns the [CardId] for an effect which can target either a stack card or
/// a character.
pub fn stack_or_character_id(targets: Option<&EffectTargets>) -> Option<CardId> {
    match targets {
        Some(EffectTargets::StackCard(stack_card_id, ..)) => Some(stack_card_id.card_id()),
        Some(EffectTargets::Character(character_id, ..)) => Some(character_id.card_id()),
        _ => None,
    }
}
