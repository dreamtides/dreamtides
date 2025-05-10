use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use tracing_macros::panic_with;

/// Returns the [CharacterId] for a [StackCardTargets::Character] target.
///
/// Panics if the target is not a character.
pub fn character_id(battle: &mut BattleState, targets: &StackCardTargets) -> CharacterId {
    match targets {
        StackCardTargets::Character(character_id) => *character_id,
        _ => {
            panic_with!("Stack card targets should be a character", battle)
        }
    }
}

/// Returns the [StackCardId] for a [StackCardTargets::StackCard] target.
///
/// Panics if the target is not a stack card.
pub fn stack_card_id(battle: &BattleState, targets: &StackCardTargets) -> StackCardId {
    match targets {
        StackCardTargets::StackCard(stack_card_id) => *stack_card_id,
        _ => {
            panic_with!("Stack card targets should be a stack card", battle)
        }
    }
}
