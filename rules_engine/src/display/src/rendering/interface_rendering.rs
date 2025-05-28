use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::numerics::Energy;
use display_data::battle_view::{ButtonView, InterfaceView};
use masonry::dimension::FlexInsets;
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;
use tracing_macros::panic_with;
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::icon;

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::display_state;
use crate::panels::panel_rendering;
use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};
use crate::rendering::labels;

pub fn interface_view(builder: &ResponseBuilder, battle: &BattleState) -> InterfaceView {
    if builder.is_for_animation() {
        return InterfaceView::default();
    }

    let current_panel_address = display_state::get_current_panel_address();
    let overlay_builder = BoxComponent::builder()
        .name("Interface Overlay")
        .style(
            FlexStyle::builder()
                .position(FlexPosition::Absolute)
                .inset(FlexInsets::all(0))
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center)
                .build(),
        )
        .child(interface_message())
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
        card_order_selector: None,
        bottom_right_button: None,
    }
}

fn primary_action_button(
    builder: &ResponseBuilder,
    battle: &BattleState,
    legal_actions: &LegalActions,
) -> Option<ButtonView> {
    if legal_actions.contains(BattleAction::SelectPromptChoice(0)) {
        let Some(PromptType::Choose { choices }) = battle.prompt.as_ref().map(|p| &p.prompt_type)
        else {
            panic_with!("Expected prompt for SelectPromptChoice action", battle);
        };
        return Some(ButtonView {
            label: labels::choice_label(choices[0].label),
            action: Some(BattleAction::SelectPromptChoice(0).into()),
        });
    }

    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, .. } = &prompt.prompt_type
    {
        let current = display_state::get_selected_energy_additional_cost().unwrap_or(*minimum);
        if legal_actions.contains(BattleAction::SelectEnergyAdditionalCost(current)) {
            return Some(ButtonView {
                label: format!("Spend {}\u{f7e4}", current),
                action: Some(BattleAction::SelectEnergyAdditionalCost(current).into()),
            });
        }
    }

    if legal_actions.contains(BattleAction::PassPriority) {
        Some(ButtonView {
            label: "Resolve".to_string(),
            action: Some(BattleAction::PassPriority.into()),
        })
    } else if legal_actions.contains(BattleAction::EndTurn) {
        Some(ButtonView {
            label: "End Turn".to_string(),
            action: Some(BattleAction::EndTurn.into()),
        })
    } else if legal_actions.contains(BattleAction::StartNextTurn) {
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
    if legal_actions.contains(BattleAction::SelectPromptChoice(1))
        && let Some(PromptType::Choose { choices }) = battle.prompt.as_ref().map(|p| &p.prompt_type)
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
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, maximum } = &prompt.prompt_type
    {
        let current = display_state::get_selected_energy_additional_cost().unwrap_or(*minimum);
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
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { minimum, .. } = &prompt.prompt_type
    {
        let current = display_state::get_selected_energy_additional_cost().unwrap_or(*minimum);
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

fn interface_message() -> impl Component {
    InterfaceMessage::builder()
        .text(format!("Note: cards drawn in excess of 10 become {} instead.", icon::ENERGY))
        .anchor_position(AnchorPosition::Top)
        .build()
}
