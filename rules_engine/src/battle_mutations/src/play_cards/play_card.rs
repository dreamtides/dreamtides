use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::StandardStaticAbility;
use battle_queries::battle_card_queries::{card, card_abilities, card_properties};
use battle_queries::card_ability_queries::effect_predicates;
use battle_queries::legal_action_queries::can_play_cards;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{
    AbilityId, CardId, CardIdType, HandCardId, StackCardId, VoidCardId,
};
use battle_state::battle_cards::ability_list::AbilityReference;
use battle_state::battle_cards::stack_card_state::{EffectTargets, SingleEffectTarget};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::card_mutations::move_card;
use crate::effects::apply_effect;
use crate::player_mutations::energy;
use crate::prompt_mutations::{add_additional_cost_prompt, add_targeting_prompt};

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
    add_targeting_prompt::execute(battle, player, stack_card_id);
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

    add_targeting_prompt::execute(battle, player, stack_card_id);
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
        let this_controller = card_properties::controller(battle, via_ability.card_id);
        let targets = if_you_do_effect_targets(
            battle,
            effect,
            via_ability.card_id,
            this_controller,
            stack_card_id,
            player,
        );
        apply_effect::execute(battle, source, effect, targets.as_ref());
    }
}

fn if_you_do_effect_targets(
    battle: &BattleState,
    effect: &Effect,
    this: CardId,
    this_controller: PlayerName,
    that: StackCardId,
    that_controller: PlayerName,
) -> Option<EffectTargets> {
    match effect {
        Effect::Effect(standard) => if_you_do_standard_effect_targets(
            battle,
            standard,
            this,
            this_controller,
            that,
            that_controller,
        ),
        Effect::WithOptions(with_options) => if_you_do_standard_effect_targets(
            battle,
            &with_options.effect,
            this,
            this_controller,
            that,
            that_controller,
        ),
        Effect::List(effects) => {
            for effect in effects {
                if let Some(targets) = if_you_do_standard_effect_targets(
                    battle,
                    &effect.effect,
                    this,
                    this_controller,
                    that,
                    that_controller,
                ) {
                    return Some(targets);
                }
            }
            None
        }
    }
}

fn if_you_do_standard_effect_targets(
    battle: &BattleState,
    effect: &StandardEffect,
    this: CardId,
    this_controller: PlayerName,
    that: StackCardId,
    that_controller: PlayerName,
) -> Option<EffectTargets> {
    let predicate = effect_predicates::get_stack_target_predicate(effect)?;
    match predicate {
        Predicate::This => to_stack_targets(battle, this, this_controller),
        Predicate::That => to_stack_targets(battle, that, that_controller),
        _ => todo!("Implement predicate {:?}", predicate),
    }
}

fn to_stack_targets(
    battle: &BattleState,
    card_id: impl CardIdType,
    controller: PlayerName,
) -> Option<EffectTargets> {
    let object_id = card::get(battle, card_id).object_id;
    Some(EffectTargets::Single(SingleEffectTarget::StackCard(
        battle.cards.to_stack_card_id(controller, card_id)?,
        object_id,
    )))
}
