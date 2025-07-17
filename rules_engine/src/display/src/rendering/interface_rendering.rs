use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::battle_card_queries::card_abilities;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::ability_list::{AbilityConfiguration, AbilityList};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{PromptData, PromptType};
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use display_data::battle_view::{ButtonView, CardOrderSelectorView, InterfaceView};
use masonry::dimension::{FlexInsets, SafeAreaInsets};
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::{BoxComponent, BoxComponentBuilder, Named};
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::icon;

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::display_state;
use crate::panels::panel_rendering;
use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};
use crate::rendering::labels;

pub fn interface_view(builder: &ResponseBuilder, battle: &BattleState) -> InterfaceView {
    let current_panel_address = display_state::get_current_panel_address(builder);

    if builder.is_for_animation() {
        let overlay_builder = overlay_builder().child(
            current_panel_address
                .map(|address| panel_rendering::render_panel(address, builder, battle)),
        );

        return InterfaceView {
            screen_overlay: overlay_builder.build().flex_node(),
            dev_button: Some(ButtonView { label: "\u{f0ad} Dev".to_string(), action: None }),
            undo_button: Some(ButtonView { label: "\u{f0e2}".to_string(), action: None }),
            ..Default::default()
        };
    }

    let overlay_builder = overlay_builder()
        .child(render_prompt_message(builder, battle))
        .child(render_hide_stack_button(builder, battle))
        .child(
            current_panel_address
                .map(|address| panel_rendering::render_panel(address, builder, battle)),
        );

    let overlay = overlay_builder.build().flex_node();
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());

    InterfaceView {
        screen_overlay: overlay,
        primary_action_button: primary_action_button(builder, battle, &legal_actions),
        primary_action_show_on_idle_duration: None,
        secondary_action_button: secondary_action_button(battle, &legal_actions),
        increment_button: increment_button(builder, battle),
        decrement_button: decrement_button(builder, battle),
        dev_button: Some(ButtonView {
            label: "\u{f0ad} Dev".to_string(),
            action: Some(BattleDisplayAction::OpenPanel(PanelAddress::Developer).into()),
        }),
        undo_button: Some(ButtonView {
            label: "\u{f0e2}".to_string(),
            action: Some(GameAction::Undo(builder.act_for_player())),
        }),
        card_order_selector: card_order_selector_view(battle),
        bottom_right_button: None,
    }
}

fn render_prompt_message(
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> Option<InterfaceMessage> {
    let prompt = battle.prompts.front()?;
    if prompt.player != builder.act_for_player() {
        return None;
    }

    let message = match get_prompt_message_from_source(battle, prompt) {
        Some(msg) => msg,
        None => get_generic_prompt_message(&prompt.prompt_type),
    };

    Some(
        InterfaceMessage::builder()
            .text(message)
            .anchor_position(AnchorPosition::Top)
            .temporary(false)
            .build(),
    )
}

fn get_prompt_message_from_source(battle: &BattleState, prompt: &PromptData) -> Option<String> {
    let card_id = prompt.source.card_id()?;
    let ability_number = match prompt.source {
        EffectSource::Event { ability_number, .. }
        | EffectSource::Triggered { ability_number, .. } => ability_number,
        EffectSource::Activated { activated_ability_id, .. } => activated_ability_id.ability_number,
        _ => return None,
    };

    let abilities = card_abilities::query(battle, card_id);
    let config = find_ability_configuration(abilities, ability_number)?;

    match &prompt.prompt_type {
        PromptType::ChooseCharacter { .. }
        | PromptType::ChooseStackCard { .. }
        | PromptType::ChooseVoidCard { .. } => config.targeting_prompt.clone(),
        PromptType::Choose { .. } => config.choice_prompt.clone(),
        PromptType::ChooseEnergyValue { .. } => config.additional_cost_prompt.clone(),
        PromptType::SelectDeckCardOrder { .. } => None,
    }
}

fn find_ability_configuration(
    abilities: &AbilityList,
    ability_number: AbilityNumber,
) -> Option<&AbilityConfiguration> {
    abilities
        .event_abilities
        .iter()
        .find(|ability_data| ability_data.ability_number == ability_number)
        .map(|ability_data| &ability_data.configuration)
        .or_else(|| {
            abilities
                .activated_abilities
                .iter()
                .find(|ability_data| ability_data.ability_number == ability_number)
                .map(|ability_data| &ability_data.configuration)
        })
        .or_else(|| {
            abilities
                .triggered_abilities
                .iter()
                .find(|ability_data| ability_data.ability_number == ability_number)
                .map(|ability_data| &ability_data.configuration)
        })
}

fn get_generic_prompt_message(prompt_type: &PromptType) -> String {
    match prompt_type {
        PromptType::ChooseCharacter { .. } => "Choose a character".to_string(),
        PromptType::ChooseStackCard { .. } => "Select a card".to_string(),
        PromptType::ChooseVoidCard { .. } => "Select a card from your void".to_string(),
        PromptType::Choose { .. } => "Select an option".to_string(),
        PromptType::ChooseEnergyValue { .. } => "Choose energy amount".to_string(),
        PromptType::SelectDeckCardOrder { .. } => "Select card position".to_string(),
    }
}

fn primary_action_button(
    builder: &ResponseBuilder,
    battle: &BattleState,
    legal_actions: &LegalActions,
) -> Option<ButtonView> {
    if legal_actions.contains(BattleAction::SelectPromptChoice(0), ForPlayer::Human) {
        let Some(PromptType::Choose { choices }) = battle.prompts.front().map(|p| &p.prompt_type)
        else {
            panic_with!("Expected prompt for SelectPromptChoice action", battle);
        };
        return Some(ButtonView {
            label: labels::choice_label(choices[0].label),
            action: Some(BattleAction::SelectPromptChoice(0).into()),
        });
    }

    if let Some(prompt) = battle.prompts.front()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, .. } = &prompt.prompt_type
    {
        let current =
            display_state::get_selected_energy_additional_cost(builder).unwrap_or(*minimum);
        if legal_actions
            .contains(BattleAction::SelectEnergyAdditionalCost(current), ForPlayer::Human)
        {
            return Some(ButtonView {
                label: format!("Spend {current}\u{f7e4}"),
                action: Some(BattleAction::SelectEnergyAdditionalCost(current).into()),
            });
        }
    }

    if legal_actions.contains(BattleAction::SubmitDeckCardOrder, ForPlayer::Human) {
        Some(ButtonView {
            label: "Submit".to_string(),
            action: Some(BattleAction::SubmitDeckCardOrder.into()),
        })
    } else if legal_actions.contains(BattleAction::PassPriority, ForPlayer::Human) {
        Some(ButtonView {
            label: "Resolve".to_string(),
            action: Some(BattleAction::PassPriority.into()),
        })
    } else if legal_actions.contains(BattleAction::EndTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: "End Turn".to_string(),
            action: Some(BattleAction::EndTurn.into()),
        })
    } else if legal_actions.contains(BattleAction::StartNextTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: "Next Turn".to_string(),
            action: Some(BattleAction::StartNextTurn.into()),
        })
    } else {
        None
    }
}

fn secondary_action_button(
    battle: &BattleState,
    legal_actions: &LegalActions,
) -> Option<ButtonView> {
    if legal_actions.contains(BattleAction::SelectPromptChoice(1), ForPlayer::Human)
        && let Some(PromptType::Choose { choices }) = battle.prompts.front().map(|p| &p.prompt_type)
        && choices.len() > 1
    {
        Some(ButtonView {
            label: labels::choice_label(choices[1].label),
            action: Some(BattleAction::SelectPromptChoice(1).into()),
        })
    } else {
        None
    }
}

fn increment_button(builder: &ResponseBuilder, battle: &BattleState) -> Option<ButtonView> {
    if let Some(prompt) = battle.prompts.front()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, maximum } = &prompt.prompt_type
    {
        let current =
            display_state::get_selected_energy_additional_cost(builder).unwrap_or(*minimum);
        return Some(ButtonView {
            label: "+1\u{f7e4}".to_string(),
            action: if current + Energy(1) <= *maximum {
                Some(
                    BattleDisplayAction::SetSelectedEnergyAdditionalCost(current + Energy(1))
                        .into(),
                )
            } else {
                None
            },
        });
    }

    None
}

fn decrement_button(builder: &ResponseBuilder, battle: &BattleState) -> Option<ButtonView> {
    if let Some(prompt) = battle.prompts.front()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, .. } = &prompt.prompt_type
    {
        let current =
            display_state::get_selected_energy_additional_cost(builder).unwrap_or(*minimum);
        return Some(ButtonView {
            label: "-1\u{f7e4}".to_string(),
            action: if current > Energy(0) && current - Energy(1) >= *minimum {
                Some(
                    BattleDisplayAction::SetSelectedEnergyAdditionalCost(current - Energy(1))
                        .into(),
                )
            } else {
                None
            },
        });
    }

    None
}

fn overlay_builder() -> BoxComponentBuilder<Named> {
    BoxComponent::builder().name("Interface Overlay").style(
        FlexStyle::builder()
            .position(FlexPosition::Absolute)
            .inset(FlexInsets::all(0))
            .align_items(FlexAlign::Center)
            .justify_content(FlexJustify::Center)
            .build(),
    )
}

fn render_hide_stack_button(
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> Option<impl Component> {
    if builder.is_for_animation() || !battle.cards.has_stack() {
        return None;
    }

    let label = if display_state::is_stack_hidden(builder) {
        icon::EYE.to_string()
    } else {
        icon::EYE_SLASH.to_string()
    };

    Some(
        BoxComponent::builder()
            .name("Hide Stack Button Container")
            .style(
                FlexStyle::builder()
                    .position(FlexPosition::Absolute)
                    .inset(SafeAreaInsets::builder().bottom(8).right(8).build())
                    .build(),
            )
            .child(
                ButtonComponent::builder()
                    .label(label)
                    .action(GameAction::BattleDisplayAction(
                        BattleDisplayAction::ToggleStackVisibility,
                    ))
                    .build(),
            )
            .build(),
    )
}

fn card_order_selector_view(battle: &BattleState) -> Option<CardOrderSelectorView> {
    if let Some(prompt) = battle.prompts.front()
        && let PromptType::SelectDeckCardOrder { .. } = &prompt.prompt_type
    {
        Some(CardOrderSelectorView { include_deck: true, include_void: true })
    } else {
        None
    }
}
