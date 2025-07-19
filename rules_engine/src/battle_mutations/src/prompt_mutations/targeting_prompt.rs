use std::collections::VecDeque;

use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::card_abilities;
use battle_queries::battle_trace;
use battle_queries::card_ability_queries::effect_predicates;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, CardId, StackCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    ChooseVoidCardPrompt, ModalEffectPrompt, OnSelected, PromptConfiguration, PromptData,
    PromptType,
};
use core_data::types::PlayerName;

/// Adds a prompt to the `battle` for targets required to play the `card_id`
/// card.
pub fn execute(battle: &mut BattleState, player: PlayerName, card_id: StackCardId) {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        let source = EffectSource::Event {
            controller: player,
            stack_card_id: card_id,
            ability_number: data.ability_number,
        };

        let mut prompts = targeting_prompts(
            battle,
            player,
            source,
            &data.ability.effect,
            None,
            OnSelected::AddStackTargets(card_id.into()),
        );

        if !prompts.is_empty() {
            battle_trace!("Adding target prompt", battle);
            battle.prompts.append(&mut prompts);
        }
    }
}

/// Adds a prompt to the `battle` for targets required for an activated ability.
pub fn execute_for_activated_ability(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
) {
    let abilities = card_abilities::query(battle, activated_ability_id.character_id);
    if let Some(ability_data) = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == activated_ability_id.ability_number)
    {
        let source = EffectSource::Activated { controller: player, activated_ability_id };
        let mut prompts = targeting_prompts(
            battle,
            player,
            source,
            &ability_data.ability.effect,
            None,
            OnSelected::AddStackTargets(activated_ability_id.into()),
        );

        if !prompts.is_empty() {
            battle_trace!("Adding target prompt for activated ability", battle);
            battle.prompts.append(&mut prompts);
        }
    }
}

/// Returns a list of prompt data for prompts required to resolve an effect, if
/// any.
pub fn targeting_prompts(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    effect: &Effect,
    that_card: Option<CardId>,
    on_selected: OnSelected,
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
        Effect::Modal(modal) => VecDeque::from([PromptData {
            source,
            player,
            prompt_type: PromptType::ModalEffect(ModalEffectPrompt {
                on_selected,
                choice_count: modal.len(),
            }),
            configuration: PromptConfiguration { optional: false },
        }]),
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
    if let Some(target_predicate) = effect_predicates::get_character_target_predicate(effect) {
        let valid =
            effect_predicates::matching_characters(battle, source, target_predicate, that_card);
        if valid.is_empty() {
            return None;
        }

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseCharacter { on_selected, valid },
            configuration: PromptConfiguration { optional },
        })
    } else if let Some(target_predicate) = effect_predicates::get_stack_target_predicate(effect) {
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
    } else if let Some(target_predicate) = effect_predicates::get_void_target_predicate(effect) {
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
