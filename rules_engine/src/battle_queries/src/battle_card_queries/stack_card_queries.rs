use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardTargets;

/// Returns the current valid stack targets for a card, if any.
pub fn targets(battle: &BattleState, card: impl CardIdType) -> Option<&StackCardTargets> {
    let stack_card = battle.cards.stack_card(StackCardId(card.card_id()))?;
    validate_targets(battle, stack_card.targets.as_ref())
}

/// Returns the provided [StackCardTargets] if they are all valid, or otherwise
/// None.
pub fn validate_targets<'a>(
    battle: &BattleState,
    targets: Option<&'a StackCardTargets>,
) -> Option<&'a StackCardTargets> {
    if targets_are_valid(battle, targets) {
        targets
    } else {
        None
    }
}

fn targets_are_valid(battle: &BattleState, targets: Option<&StackCardTargets>) -> bool {
    match targets {
        Some(StackCardTargets::Character(character_id, object_id)) => {
            battle.cards.is_valid_object_id(character_id.card_id(), *object_id)
        }
        Some(StackCardTargets::StackCard(stack_card_id, object_id)) => {
            battle.cards.is_valid_object_id(stack_card_id.card_id(), *object_id)
        }
        None => true,
    }
}
