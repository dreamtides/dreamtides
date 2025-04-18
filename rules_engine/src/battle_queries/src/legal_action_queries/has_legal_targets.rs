use ability_data::ability::Ability;
use ability_data::effect::Effect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::CardIdType;
use core_data::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::predicate_queries::predicates;

/// Returns true if the given card has legal targets for its event abilities.
pub fn for_event(battle: &BattleData, source: EffectSource, card_id: impl CardIdType) -> bool {
    let Some(card) = battle.cards.card(card_id) else {
        return false;
    };

    let controller = card.controller();
    for ability in &card.abilities {
        if let Ability::Event(effect) = ability {
            if !has_legal_targets_for_effect(battle, controller, source, effect) {
                return false;
            }
        }
    }

    true
}

fn has_legal_targets_for_effect(
    battle: &BattleData,
    controller: PlayerName,
    source: EffectSource,
    effect: &Effect,
) -> bool {
    match effect {
        Effect::Effect(standard_effect) => {
            if let Some(predicate) = predicates::get_target_predicate(standard_effect) {
                !predicates::matching_characters(battle, controller, source, predicate).is_empty()
            } else {
                true
            }
        }
        Effect::WithOptions(effect_with_options) => {
            if let Some(predicate) = predicates::get_target_predicate(&effect_with_options.effect) {
                !predicates::matching_characters(battle, controller, source, predicate).is_empty()
            } else {
                true
            }
        }
        Effect::List(effects) => effects.iter().all(|effect_with_options| {
            if let Some(predicate) = predicates::get_target_predicate(&effect_with_options.effect) {
                !predicates::matching_characters(battle, controller, source, predicate).is_empty()
            } else {
                true
            }
        }),
    }
}
