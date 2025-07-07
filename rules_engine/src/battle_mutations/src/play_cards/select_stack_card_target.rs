use battle_queries::battle_card_queries::card;
use battle_queries::panic_with;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

/// Selects a character as the target of a card on the stack.
pub fn character(battle: &mut BattleState, player: PlayerName, character_id: CharacterId) {
    let object_id = card::get(battle, character_id).object_id;
    let Some(stack_card) = battle.cards.top_of_stack_mut() else {
        panic_with!("No active stack", battle);
    };
    stack_card.targets = Some(EffectTargets::Character(character_id, object_id));
    battle.prompt = None;
    let source_id = stack_card.id;
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
        player,
        source_id,
        targets: EffectTargets::Character(character_id, object_id),
    });
}

/// Selects a card on the stack as a target of another card on the stack.
pub fn on_stack(battle: &mut BattleState, player: PlayerName, stack_card_id: StackCardId) {
    let object_id = card::get(battle, stack_card_id).object_id;
    let Some(stack_card) = battle.cards.top_of_stack_mut() else {
        panic_with!("No active stack", battle);
    };
    stack_card.targets = Some(EffectTargets::StackCard(stack_card_id, object_id));
    battle.prompt = None;
    let source_id = stack_card.id;
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
        player,
        source_id,
        targets: EffectTargets::StackCard(stack_card_id, object_id),
    });
}
