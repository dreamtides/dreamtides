use ability_data::effect::ModelEffectChoiceIndex;
use action_data::game_action_data::GameAction;
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

    let descriptions = modal_effect_descriptions(&card_rendering::rules_text(battle, card_id));
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
    let view = TokenCardView::builder()
        .id(adapter::modal_effect_choice_client_id(card_id, index))
        .image(card_rendering::card_image(battle, card_id))
        .name(format!("Choice {}", index.value() + 1))
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
        .rules_text(description.to_string())
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
pub fn modal_effect_descriptions(rules_text: &str) -> Vec<String> {
    let mut descriptions = Vec::new();
    let mut current_pos = 0;

    while let Some(start_tag) = rules_text[current_pos..].find("<indent") {
        let indent_start = current_pos + start_tag;
        if let Some(close_bracket) = rules_text[indent_start..].find(">") {
            let content_start = indent_start + close_bracket + 1;
            if let Some(end_tag) = rules_text[content_start..].find("</indent>") {
                let content_end = content_start + end_tag;
                let raw_description = &rules_text[content_start..content_end];
                let clean_description = clean_modal_description(raw_description);
                descriptions.push(clean_description);
                current_pos = content_end + 8;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    descriptions
}

fn clean_modal_description(raw_description: &str) -> String {
    let mut result = raw_description.to_string();

    // Remove HTML color and bold tags with cost information
    // Pattern: <color=#XXXXXX><b>COST</b></color>:
    while let Some(color_start) = result.find("<color=") {
        if let Some(color_end) = result[color_start..].find("</color>:") {
            let full_pattern_end = color_start + color_end + 9; // "</color>:" is 9 chars
            result.replace_range(color_start..full_pattern_end, "");
        } else {
            break;
        }
    }

    // Clean up any remaining HTML tags
    while let Some(tag_start) = result.find('<') {
        if let Some(tag_end) = result[tag_start..].find('>') {
            result.replace_range(tag_start..tag_start + tag_end + 1, "");
        } else {
            break;
        }
    }

    result.trim().to_string()
}
