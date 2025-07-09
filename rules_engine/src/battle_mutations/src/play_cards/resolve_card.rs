use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_queries::{assert_that, battle_trace, panic_with};
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, CardIdType, CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::{StackItemId, StackItemState};
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;
use core_data::types::PlayerName;

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

    resolve_stack_item(battle, &stack_card);

    // After a card resolves, its controller receives priority (if the stack is
    // not empty)
    battle.stack_priority = battle.cards.has_stack().then_some(stack_card.controller);
}

fn resolve_stack_item(battle: &mut BattleState, item: &StackItemState) {
    match item.id {
        StackItemId::Card(card_id) => resolve_stack_card(battle, item, card_id),
        StackItemId::ActivatedAbility(ability_id) => {
            resolve_activated_ability(battle, item, ability_id)
        }
    }
}

fn resolve_stack_card(battle: &mut BattleState, item: &StackItemState, card_id: StackCardId) {
    battle_trace!("Resolving card", battle, card_id = card_id);
    if card_properties::card_type(battle, card_id) == CardType::Event {
        let source = EffectSource::Game { controller: item.controller };
        apply_event_effects(battle, item, card_id);
        move_card::from_stack_to_void(battle, source, item.controller, card_id);
    } else {
        let character_id = CharacterId(card_id.card_id());
        let source = EffectSource::Character { controller: item.controller, character_id };
        battle.push_animation(source, || BattleAnimation::ResolveCharacter { character_id });
        character_limit::apply(battle, source, item.controller);
        move_card::from_stack_to_battlefield(battle, source, item.controller, card_id);
    }
}

fn apply_event_effects(battle: &mut BattleState, item: &StackItemState, card_id: StackCardId) {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        let event_source = EffectSource::Event {
            controller: item.controller,
            stack_card_id: card_id,
            ability_number: data.ability_number,
        };
        apply_effect::execute(battle, event_source, &data.ability.effect, item.targets.as_ref());
    }
}

fn resolve_activated_ability(
    battle: &mut BattleState,
    item: &StackItemState,
    ability_id: ActivatedAbilityId,
) {
    todo!()
}
