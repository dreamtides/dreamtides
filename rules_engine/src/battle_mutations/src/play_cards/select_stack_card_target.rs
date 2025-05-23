use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use core_data::types::PlayerName;
use tracing_macros::panic_with;

/// Selects a character as the target of a card on the stack.
pub fn character(battle: &mut BattleState, player: PlayerName, character_id: CharacterId) {
    let Some(stack_card) = battle.cards.top_of_stack_mut() else {
        panic_with!("No active stack", battle);
    };
    stack_card.targets = Some(StackCardTargets::Character(character_id));
    battle.prompt = None;
    let source_id = stack_card.id;
    battle.push_animation(|| BattleAnimation::SelectStackCardTargets {
        player,
        source_id,
        targets: StackCardTargets::Character(character_id),
    });
}

/// Selects a card on the stack as a target of another card on the stack.
pub fn on_stack(battle: &mut BattleState, player: PlayerName, stack_card_id: StackCardId) {
    let Some(stack_card) = battle.cards.top_of_stack_mut() else {
        panic_with!("No active stack", battle);
    };
    stack_card.targets = Some(StackCardTargets::StackCard(stack_card_id));
    battle.prompt = None;
    let source_id = stack_card.id;
    battle.push_animation(|| BattleAnimation::SelectStackCardTargets {
        player,
        source_id,
        targets: StackCardTargets::StackCard(stack_card_id),
    });
}
