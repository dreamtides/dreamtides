use ability_data::ability::Ability;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::{CardIdType, StackCardId};
use core_data::identifiers::AbilityNumber;

use crate::predicate_queries::effect_predicates;

/// Returns true if the given card has legal targets for its event abilities.
pub fn for_event(battle: &BattleData, card_id: impl CardIdType) -> bool {
    let Some(card) = battle.cards.card(card_id) else {
        return false;
    };

    let controller = card.controller();
    for (ability_index, ability) in card.abilities.iter().enumerate() {
        if let Ability::Event(event) = ability {
            let source = EffectSource::Event {
                controller,
                card: StackCardId(card_id.card_id()),
                ability_number: AbilityNumber(ability_index),
            };
            if !has_legal_targets_for_effect(battle, source, &event.effect) {
                return false;
            }
        }
    }

    true
}

fn has_legal_targets_for_effect(
    battle: &BattleData,
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
    }
}

fn has_legal_targets_for_standard_effect(
    battle: &BattleData,
    source: EffectSource,
    effect: &StandardEffect,
) -> bool {
    if let Some(predicate) = effect_predicates::get_character_target_predicate(effect) {
        !effect_predicates::matching_characters(battle, source, predicate).is_empty()
    } else if let Some(predicate) = effect_predicates::get_stack_target_predicate(effect) {
        !effect_predicates::matching_cards_on_stack(battle, source, predicate).is_empty()
    } else {
        true
    }
}
