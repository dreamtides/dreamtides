use ability_data::ability::{DisplayedAbility, DisplayedAbilityEffect, DisplayedModalEffectChoice};
use ability_data::effect::ModelEffectChoiceIndex;
use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::display_color;
use core_data::numerics::Energy;
use display_data::card_view::{CardActions, CardView};
use display_data::object_position::{ObjectPosition, Position};
use fluent::fluent_args;
use tabula_data::localized_strings::StringContext;
use tabula_ids::string_id;

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::token_rendering::TokenCardView;
use crate::rendering::{card_rendering, token_rendering};

/// [CardView]s for cards representing the choices in an active modal effect
/// prompt, if any.
pub fn cards(builder: &ResponseBuilder, battle: &BattleState) -> Vec<CardView> {
    let Some(prompt) = battle.prompts.front() else {
        return vec![];
    };
    let PromptType::ModalEffect(modal) = &prompt.prompt_type else {
        return vec![];
    };
    let Some(card_id) = prompt.source.card_id() else {
        return vec![];
    };
    if prompt.player != builder.act_for_player() {
        return vec![];
    }

    let definition = card::get_definition(battle, card_id);
    let descriptions = modal_effect_descriptions(builder, &definition.displayed_abilities);
    modal
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            modal_effect_card_view(
                builder,
                battle,
                card_id,
                choice.energy_cost,
                ModelEffectChoiceIndex(index),
                &descriptions[index],
            )
        })
        .collect()
}

fn modal_effect_card_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
    cost: Energy,
    index: ModelEffectChoiceIndex,
    description: &str,
) -> CardView {
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    let select_action = BattleAction::SelectModalEffectChoice(index);
    let can_select = legal_actions.contains(select_action, ForPlayer::Human);
    let formatted = description.to_string();
    let view = TokenCardView::builder()
        .id(adapter::modal_effect_choice_client_id(card_id, index))
        .image(card_rendering::card_image(battle, card_id))
        .name(builder.string_with_args(
            string_id::MODAL_EFFECT_CHOICE_CARD_NAME,
            fluent_args!("number" => index.value() + 1),
        ))
        .position(ObjectPosition { position: Position::Browser, sorting_key: index.value() as u32 })
        .create_position(ObjectPosition {
            position: Position::HiddenWithinCard(adapter::client_card_id(card_id)),
            sorting_key: index.value() as u32,
        })
        .destroy_position(ObjectPosition {
            position: Position::HiddenWithinCard(adapter::client_card_id(card_id)),
            sorting_key: index.value() as u32,
        })
        .cost(cost.to_string())
        .rules_text(formatted)
        .outline_color(if can_select { display_color::GREEN } else { display_color::WHITE })
        .actions(CardActions {
            on_click: can_select.then_some(GameAction::BattleAction(select_action)),
            ..CardActions::default()
        })
        .build();

    token_rendering::token_card_view(view)
}

/// [String]s for the descriptions of the choices in an active modal effect
/// prompt, if any.
pub fn modal_effect_descriptions(
    builder: &ResponseBuilder,
    abilities: &[DisplayedAbility],
) -> Vec<String> {
    all_modal_effect_descriptions(abilities)
        .iter()
        .map(|choice| {
            builder.tabula().strings.format_display_string(
                &choice.effect,
                StringContext::CardText,
                fluent_args![],
            )
        })
        .collect()
}

fn all_modal_effect_descriptions(
    abilities: &[DisplayedAbility],
) -> Vec<DisplayedModalEffectChoice> {
    let mut result = vec![];
    for ability in abilities {
        match ability {
            DisplayedAbility::Event { event } => {
                result.extend(effect_modal_effect_descriptions(&event.effect));
            }
            DisplayedAbility::Activated { effect, .. } => {
                result.extend(effect_modal_effect_descriptions(effect));
            }
            _ => {}
        }
    }
    result
}

fn effect_modal_effect_descriptions(
    effect: &DisplayedAbilityEffect,
) -> Vec<DisplayedModalEffectChoice> {
    if let DisplayedAbilityEffect::Modal(choices) = effect { choices.clone() } else { vec![] }
}
