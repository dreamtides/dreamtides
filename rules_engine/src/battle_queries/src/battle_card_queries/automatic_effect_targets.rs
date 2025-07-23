use std::collections::{BTreeSet, VecDeque};

use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StandardEffectTarget, VoidCardTarget,
};
use battle_state::core::effect_source::EffectSource;

use crate::battle_card_queries::card;
use crate::card_ability_queries::{effect_predicates, effect_queries, target_predicates};

pub enum AutomaticEffectTargets {
    RequiresPrompt,
    Targets(Option<EffectTargets>),
}

/// Attempts to automatically select targets for an effect.
///
/// If all of the targets needed to resolve this effect can be selected
/// automatically without user input, returns targets. Otherwise returns
/// `RequiresPrompt` to indicate that the effect requires user input.
pub fn query(
    battle: &BattleState,
    source: EffectSource,
    effect: &Effect,
    that_card: Option<CardId>,
    modal_choice: Option<ModelEffectChoiceIndex>,
) -> AutomaticEffectTargets {
    match effect {
        Effect::Effect(standard_effect) => {
            if let Some(targets) =
                standard_effect_automatic_targets(battle, source, standard_effect, that_card)
            {
                AutomaticEffectTargets::Targets(Some(EffectTargets::Standard(targets)))
            } else if target_predicates::has_targets(standard_effect) {
                AutomaticEffectTargets::RequiresPrompt
            } else {
                AutomaticEffectTargets::Targets(None)
            }
        }
        Effect::WithOptions(with_options) => {
            if let Some(targets) =
                standard_effect_automatic_targets(battle, source, &with_options.effect, that_card)
            {
                AutomaticEffectTargets::Targets(Some(EffectTargets::Standard(targets)))
            } else if target_predicates::has_targets(&with_options.effect) {
                AutomaticEffectTargets::RequiresPrompt
            } else {
                AutomaticEffectTargets::Targets(None)
            }
        }
        Effect::List(effects) => {
            let mut target_list = VecDeque::new();
            for effect_item in effects {
                if let Some(targets) = standard_effect_automatic_targets(
                    battle,
                    source,
                    &effect_item.effect,
                    that_card,
                ) {
                    target_list.push_back(Some(targets));
                } else if target_predicates::has_targets(&effect_item.effect) {
                    return AutomaticEffectTargets::RequiresPrompt;
                } else {
                    target_list.push_back(None);
                }
            }
            AutomaticEffectTargets::Targets(Some(EffectTargets::EffectList(target_list)))
        }
        Effect::Modal(modal) => {
            if let Some(modal_choice) = modal_choice {
                query(battle, source, &modal[modal_choice.value()].effect, that_card, None)
            } else {
                AutomaticEffectTargets::RequiresPrompt
            }
        }
    }
}

fn standard_effect_automatic_targets(
    battle: &BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    that_card: Option<CardId>,
) -> Option<StandardEffectTarget> {
    if let Some(target_predicate) = target_predicates::get_character_target_predicate(effect) {
        let valid = effect_predicates::matching_characters(
            battle,
            source,
            target_predicate,
            that_card,
            effect_queries::character_targeting_flags(effect),
        );
        if valid.len() == 1 {
            let character_id = valid.iter().next().unwrap();
            let object_id = card::get(battle, character_id).object_id;
            Some(StandardEffectTarget::Character(character_id, object_id))
        } else {
            None
        }
    } else if let Some(target_predicate) = target_predicates::get_stack_target_predicate(effect) {
        let valid =
            effect_predicates::matching_cards_on_stack(battle, source, target_predicate, that_card);
        if valid.len() == 1 {
            let stack_card_id = valid.iter().next().unwrap();
            let object_id = card::get(battle, stack_card_id).object_id;
            Some(StandardEffectTarget::StackCard(stack_card_id, object_id))
        } else {
            None
        }
    } else if let Some(target_predicate) = target_predicates::get_void_target_predicate(effect) {
        let valid =
            effect_predicates::matching_cards_in_void(battle, source, target_predicate, that_card);
        if valid.len() == 1 {
            let void_card_id = valid.iter().next().unwrap();
            let object_id = card::get(battle, void_card_id).object_id;
            let mut void_targets = BTreeSet::new();
            void_targets.insert(VoidCardTarget { id: void_card_id, object_id });
            Some(StandardEffectTarget::VoidCardSet(void_targets))
        } else {
            None
        }
    } else {
        None
    }
}
