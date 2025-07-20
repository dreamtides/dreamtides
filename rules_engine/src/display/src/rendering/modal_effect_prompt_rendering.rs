use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_abilities};
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{AbilityId, CardId};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::display_color;
use core_data::numerics::Energy;
use display_data::card_view::CardView;
use display_data::object_position::{ObjectPosition, Position};

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::card_rendering;

/// [CardView]s for cards representing the choices in an active modal effect
/// prompt, if any.
pub fn cards(builder: &ResponseBuilder, battle: &BattleState) -> Vec<CardView> {
    let Some(prompt) = battle.prompts.front() else {
        return vec![];
    };
    let PromptType::ModalEffect(_modal) = &prompt.prompt_type else {
        return vec![];
    };
    let Some(card_id) = prompt.source.card_id() else {
        return vec![];
    };
    let Some(ability_number) = prompt.source.ability_number() else {
        return vec![];
    };
    let ability = card_abilities::ability(battle, AbilityId { card_id, ability_number });
    let Some(Effect::Modal(choices)) = ability.effect() else {
        panic_with!("Expected modal effect", battle, card_id);
    };
    let descriptions = modal_effect_descriptions(&card_rendering::rules_text(battle, card_id));
    choices
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
    let context = CardViewContext::Battle(battle, card::get(battle, card_id).name, card_id);
    let mut base_view = card_rendering::card_view(builder, &context);
    base_view.id = adapter::modal_effect_choice_client_id(card_id, index);
    base_view.position =
        ObjectPosition { position: Position::Browser, sorting_key: index.value() as u32 };
    base_view.backless = true;
    base_view.create_position = if index.value() == 0 {
        None
    } else {
        Some(ObjectPosition {
            position: Position::HiddenWithinCard(adapter::client_card_id(card_id)),
            sorting_key: index.value() as u32,
        })
    };
    base_view.destroy_position = if index.value() == 0 {
        None
    } else {
        Some(ObjectPosition {
            position: Position::HiddenWithinCard(adapter::client_card_id(card_id)),
            sorting_key: index.value() as u32,
        })
    };

    if let Some(revealed) = &mut base_view.revealed {
        let legal_actions = legal_actions::compute(battle, builder.act_for_player());
        let select_action = BattleAction::SelectModalEffectChoice(index);
        let can_select = legal_actions.contains(select_action, ForPlayer::Human);

        revealed.cost = Some(cost.to_string());
        revealed.card_type = "Choice".to_string();
        revealed.rules_text = description.to_string();
        revealed.outline_color =
            if can_select { Some(display_color::GREEN) } else { Some(display_color::WHITE) };
        revealed.actions.can_play = None;
        revealed.actions.on_click = Some(GameAction::BattleAction(select_action));
    }

    base_view
}

fn modal_effect_descriptions(rules_text: &str) -> Vec<String> {
    let mut descriptions = Vec::new();
    let mut current_pos = 0;

    while let Some(start_tag) = rules_text[current_pos..].find("<indent") {
        let indent_start = current_pos + start_tag;
        if let Some(close_bracket) = rules_text[indent_start..].find(">") {
            let content_start = indent_start + close_bracket + 1;
            if let Some(end_tag) = rules_text[content_start..].find("</indent>") {
                let content_end = content_start + end_tag;
                let description = rules_text[content_start..content_end].trim().to_string();
                descriptions.push(description);
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
