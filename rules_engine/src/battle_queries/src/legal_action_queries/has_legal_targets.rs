use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, StackCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::battle_card_queries::card;
use crate::card_ability_queries::{effect_predicates, effect_queries, target_predicates};

/// Returns true if the given card has legal targets for its event abilities, or
/// if this card does not require targets.
pub fn for_event(battle: &BattleState, controller: PlayerName, card_id: CardId) -> bool {
    for data in &card::ability_list(battle, card_id).event_abilities {
        let source = EffectSource::Event {
            controller,
            stack_card_id: StackCardId(card_id),
            ability_number: data.ability_number,
        };

        if !has_legal_targets_for_effect(battle, source, &data.ability.effect) {
            return false;
        }
    }

    true
}

/// Returns true if the given effect has legal targets for the given source.
pub fn has_legal_targets_for_effect(
    battle: &BattleState,
    source: EffectSource,
    effect: &Effect,
) -> bool {
    match effect {
        Effect::Effect(standard_effect) => {
            has_legal_targets_for_standard_effect(battle, source, standard_effect)
        }
        Effect::WithOptions(effect_with_options) => {
            has_legal_targets_for_standard_effect(battle, source, &effect_with_options.effect)
        }
        Effect::List(effects) => effects.iter().all(|effect_with_options| {
            has_legal_targets_for_standard_effect(battle, source, &effect_with_options.effect)
        }),
        Effect::ListWithOptions(list_with_options) => {
            list_with_options.effects.iter().all(|effect_with_options| {
                has_legal_targets_for_standard_effect(battle, source, &effect_with_options.effect)
            })
        }
        Effect::Modal(modal) => {
            modal.iter().any(|choice| has_legal_targets_for_effect(battle, source, &choice.effect))
        }
    }
}

fn has_legal_targets_for_standard_effect(
    battle: &BattleState,
    source: EffectSource,
    effect: &StandardEffect,
) -> bool {
    if let Some(predicate) = target_predicates::get_character_target_predicate(effect) {
        !effect_predicates::matching_characters(
            battle,
            source,
            predicate,
            None,
            effect_queries::character_targeting_flags(effect),
        )
        .is_empty()
    } else if let Some(predicate) = target_predicates::get_stack_target_predicate(effect) {
        !effect_predicates::matching_cards_on_stack(battle, source, predicate, None).is_empty()
    } else if let Some(predicate) = target_predicates::get_void_target_predicate(effect) {
        !effect_predicates::matching_cards_in_void(battle, source, predicate, None).is_empty()
    } else {
        true
    }
}
