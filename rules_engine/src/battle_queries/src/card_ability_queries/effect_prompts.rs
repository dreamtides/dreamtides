use std::collections::VecDeque;

use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    ChooseVoidCardPrompt, ModalEffectPrompt, OnSelected, PromptConfiguration, PromptData,
    PromptType,
};
use core_data::types::PlayerName;

use crate::card_ability_queries::{effect_predicates, effect_queries, target_predicates};

/// Returns a list of [PromptData] for prompts required to resolve an effect, if
/// any.
///
/// # Arguments
///
/// * `battle` - The current battle state
/// * `player` - The player who is resolving the effect
/// * `source` - The source of the effect
/// * `effect` - The effect to resolve
/// * `that_card` - The card which should be used to interpret the word "that"
///   in resolving the effect, e.g. for triggered effects.
/// * `on_selected` - The action to take when a target is selected
/// * `modal_choice` - The index of the modal choice to resolve, if one has
///   already been selected.
/// * `ability_number` - The ability number of the ability which created this
///   effect.
pub fn query(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    effect: &Effect,
    that_card: Option<CardId>,
    on_selected: OnSelected,
    modal_choice: Option<ModelEffectChoiceIndex>,
) -> VecDeque<PromptData> {
    match effect {
        Effect::Effect(standard_effect) => standard_effect_targeting_prompt(
            battle,
            player,
            source,
            standard_effect,
            false,
            that_card,
            on_selected,
        )
        .map(|prompt_data| VecDeque::from([prompt_data]))
        .unwrap_or_default(),
        Effect::WithOptions(with_options) => standard_effect_targeting_prompt(
            battle,
            player,
            source,
            &with_options.effect,
            with_options.optional,
            that_card,
            on_selected,
        )
        .map(|prompt_data| VecDeque::from([prompt_data]))
        .unwrap_or_default(),
        Effect::List(effects) => effects
            .iter()
            .filter_map(|effect| {
                standard_effect_targeting_prompt(
                    battle,
                    player,
                    source,
                    &effect.effect,
                    effect.optional,
                    that_card,
                    on_selected,
                )
            })
            .collect(),
        Effect::Modal(modal) => {
            if let Some(choice) = modal_choice {
                query(
                    battle,
                    player,
                    source,
                    &modal[choice.value()].effect,
                    that_card,
                    on_selected,
                    None,
                )
            } else {
                VecDeque::from([PromptData {
                    source,
                    player,
                    prompt_type: PromptType::ModalEffect(ModalEffectPrompt {
                        on_selected,
                        choice_count: modal.len(),
                        pay_energy: modal.iter().map(|choice| choice.energy_cost).collect(),
                    }),
                    configuration: PromptConfiguration { optional: false },
                }])
            }
        }
    }
}

/// Creates a prompt data for a standard effect if it requires target selection.
///
/// Returns the prompt data if targets are required and there are legal targets
/// available and None otherwise.
fn standard_effect_targeting_prompt(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    effect: &StandardEffect,
    optional: bool,
    that_card: Option<CardId>,
    on_selected: OnSelected,
) -> Option<PromptData> {
    if let Some(target_predicate) = target_predicates::get_character_target_predicate(effect) {
        let valid = effect_predicates::matching_characters(
            battle,
            source,
            target_predicate,
            that_card,
            effect_queries::character_targeting_flags(effect),
        );
        if valid.is_empty() {
            return None;
        }

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseCharacter { on_selected, valid },
            configuration: PromptConfiguration { optional },
        })
    } else if let Some(target_predicate) = target_predicates::get_stack_target_predicate(effect) {
        let valid =
            effect_predicates::matching_cards_on_stack(battle, source, target_predicate, that_card);
        if valid.is_empty() {
            return None;
        }

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseStackCard { on_selected, valid },
            configuration: PromptConfiguration { optional },
        })
    } else if let Some(target_predicate) = target_predicates::get_void_target_predicate(effect) {
        let valid =
            effect_predicates::matching_cards_in_void(battle, source, target_predicate, that_card);
        if valid.is_empty() {
            return None;
        }

        let maximum_selection = match effect {
            StandardEffect::ReturnFromYourVoidToHand { .. } => 1,
            StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => *count,
            _ => todo!("Implement support for predicate: {target_predicate:?}"),
        };

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseVoidCard(ChooseVoidCardPrompt {
                on_selected,
                valid,
                selected: CardSet::default(),
                maximum_selection,
            }),
            configuration: PromptConfiguration { optional },
        })
    } else {
        None
    }
}
