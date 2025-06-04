use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardTargets;

/// Returns the [CharacterId] for a [StackCardTargets::Character] target.
pub fn character_id(targets: Option<&StackCardTargets>) -> Option<CharacterId> {
    match targets {
        Some(StackCardTargets::Character(character_id, ..)) => Some(*character_id),
        _ => None,
    }
}

/// Returns the [StackCardId] for a [StackCardTargets::StackCard] target.
pub fn stack_card_id(targets: Option<&StackCardTargets>) -> Option<StackCardId> {
    match targets {
        Some(StackCardTargets::StackCard(stack_card_id, ..)) => Some(*stack_card_id),
        _ => None,
    }
}
