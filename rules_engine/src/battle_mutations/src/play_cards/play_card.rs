use ability_data::static_ability::StandardStaticAbility;
use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_queries::legal_action_queries::can_play_cards;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{AbilityId, CardIdType, HandCardId, StackCardId, VoidCardId};
use battle_state::battle_cards::ability_list::AbilityReference;
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::card_mutations::move_card;
use crate::effects::apply_effect_prompt_for_targets;
use crate::player_mutations::energy;
use crate::prompt_mutations::{add_additional_cost_prompt, targeting_prompt};

/// Plays a card to the stack as `player` by paying its costs. If the
/// card requires targets or choices, a prompt will be displayed.
pub fn from_hand(battle: &mut BattleState, player: PlayerName, card_id: HandCardId) {
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::PlayCard {
        player,
        card_id: card_id.card_id(),
        from_zone: Zone::Hand,
    });

    if let Some(cost) = card_properties::energy_cost(battle, card_id) {
        energy::spend(battle, player, source, cost);
    }
    let stack_card_id = move_card::from_hand_to_stack(battle, source, player, card_id);

    // Opponent gets priority when a card is played
    battle.stack_priority = Some(player.opponent());
    targeting_prompt::execute(battle, player, stack_card_id);
    add_additional_cost_prompt::execute(battle, player, stack_card_id);
    battle.push_animation(source, || BattleAnimation::PlayedCard {
        player,
        card_id: stack_card_id.card_id(),
        from_zone: Zone::Stack,
    });
    battle.triggers.push(source, Trigger::PlayedCardFromHand(stack_card_id));
}

/// Plays a card from the void to the stack as `player` by paying its costs. If
/// the card requires targets or choices, a prompt will be displayed.
pub fn from_void(
    battle: &mut BattleState,
    player: PlayerName,
    card_id: VoidCardId,
    via_ability: AbilityId,
) {
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::PlayCard {
        player,
        card_id: card_id.card_id(),
        from_zone: Zone::Void,
    });

    energy::spend(
        battle,
        player,
        source,
        can_play_cards::play_from_void_energy_cost(battle, card_id, via_ability),
    );

    let stack_card_id = move_card::from_void_to_stack(battle, source, player, card_id);

    battle.stack_priority = Some(player.opponent());

    apply_if_you_do_effect(battle, player, stack_card_id, via_ability);

    targeting_prompt::execute(battle, player, stack_card_id);
    add_additional_cost_prompt::execute(battle, player, stack_card_id);
    battle.push_animation(source, || BattleAnimation::PlayedCard {
        player,
        card_id: stack_card_id.card_id(),
        from_zone: Zone::Stack,
    });
    battle.triggers.push(source, Trigger::PlayedCardFromVoid(stack_card_id));
}

fn apply_if_you_do_effect(
    battle: &mut BattleState,
    player: PlayerName,
    stack_card_id: StackCardId,
    via_ability: AbilityId,
) {
    let source = EffectSource::IfYouDo { controller: player, ability_id: via_ability };
    let ability = card_abilities::ability(battle, via_ability);
    if let AbilityReference::Static(static_ability) = ability
        && let StandardStaticAbility::PlayFromVoid(play) = static_ability.standard_static_ability()
        && let Some(effect) = &play.if_you_do
    {
        apply_effect_prompt_for_targets::execute(
            battle,
            source,
            effect,
            Some(stack_card_id.card_id()),
        );
    }
}
