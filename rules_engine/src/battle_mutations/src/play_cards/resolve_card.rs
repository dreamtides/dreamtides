use ability_data::ability::Ability;
use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardState;
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;
use core_data::types::PlayerName;
use tracing_macros::{assert_that, battle_trace, panic_with};

use crate::effects::apply_effect;

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
    if card_properties::card_type(battle, card.id) == CardType::Event {
        apply_event_effects(battle, card);
    }
}

fn apply_event_effects(battle: &mut BattleState, card: &StackCardState) {
    for (ability_number, ability) in card_abilities::query(battle, card.id) {
        if let Ability::Event(event) = ability {
            let event_source = EffectSource::Event {
                controller: card.controller,
                stack_card_id: card.id,
                ability_number: *ability_number,
            };
            apply_effect::execute(battle, event_source, &event.effect, &card.targets);
        }
    }
}
