use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use tracing_macros::panic_with;

/// Returns the [CharacterId] for a [StackCardTargets::Character] target.
///
/// Panics if the target is not a character.
pub fn character_id(battle: &mut BattleState, targets: &Option<StackCardTargets>) -> CharacterId {
    match targets {
        Some(StackCardTargets::Character(character_id)) => *character_id,
        _ => {
            panic_with!("Card targets should be a character", battle, targets)
        }
    }
}

/// Returns the [StackCardId] for a [StackCardTargets::StackCard] target.
///
/// Panics if the target is not a stack card.
pub fn stack_card_id(battle: &BattleState, targets: &Option<StackCardTargets>) -> StackCardId {
    match targets {
        Some(StackCardTargets::StackCard(stack_card_id)) => *stack_card_id,
        _ => {
            panic_with!("Card targets should be a stack card", battle, targets)
        }
    }
}
