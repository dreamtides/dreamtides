use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardState;
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;
use core_data::types::PlayerName;
use tracing_macros::{assert_that, battle_trace, panic_with};

use crate::card_mutations::move_card;
use crate::effects::apply_effect;
use crate::play_cards::character_limit;

/// Marks a player as having taken the "pass" action on the current stack.
///
/// Card resolution works as follows in Dreamtides: A player may play a card
/// (during their turn or at the end of their opponent's turn), which creates a
/// "stack", which is resolved via a system of "priority":
///
/// - Whenever a card is played, the opponent of the card's controller receives
///   priority.
/// - Whenever a card resolves, the controller of that card receives priority if
///   the stack is not empty.
///
/// Priority cannot be "held", and players cannot add more than one card to the
/// stack at a time.
///
/// When a player has priority, they may either play a card or take the "pass"
/// action. If the player with priority takes the "pass" action, the top card of
/// the stack resolves. Note that only *one* player is ever required to pass
/// priority to resolve a card on the stack.
///
/// Cards resolve in Last In, First Out order, meaning the top card of the stack
/// is resolved first.
pub fn pass_priority(battle: &mut BattleState, player: PlayerName) {
    assert_that!(
        battle.stack_priority == Some(player),
        "Player does not have priority",
        battle,
        player
    );

    let Some(stack_card) = battle.cards.top_of_stack().cloned() else {
        panic_with!("No cards on stack", battle);
    };

    resolve_card(battle, &stack_card);

    // After a card resolves, its controller receives priority (if the stack is
    // not empty)
    battle.stack_priority = battle.cards.has_stack().then_some(stack_card.controller);
}

fn resolve_card(battle: &mut BattleState, card: &StackCardState) {
    battle_trace!("Resolving card", battle, card_id = card.id);
    let source = EffectSource::Game { controller: card.controller };
    if card_properties::card_type(battle, card.id) == CardType::Event {
        apply_event_effects(battle, card);
        move_card::from_stack_to_void(battle, source, card.controller, card.id);
    } else {
        character_limit::apply(battle, source, card.controller);
        move_card::from_stack_to_battlefield(battle, source, card.controller, card.id);
    }
}

fn apply_event_effects(battle: &mut BattleState, card: &StackCardState) {
    for (ability_number, ability) in &card_abilities::query(battle, card.id).event_abilities {
        let event_source = EffectSource::Event {
            controller: card.controller,
            stack_card_id: card.id,
            ability_number: *ability_number,
        };
        apply_effect::execute(battle, event_source, &ability.effect, &card.targets);
    }
}
