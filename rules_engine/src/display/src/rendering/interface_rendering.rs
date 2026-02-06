use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::core::effect_source::CardSource;
use battle_state::prompt_types::prompt_data::{PromptData, PromptType};
use core_data::numerics::Energy;
use display_data::battle_view::{
    ButtonView, CardBrowserView, CardOrderSelectorView, InterfaceView,
};
use fluent::FluentArgs;
use masonry::dimension::{FlexInsets, SafeAreaInsets};
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;
use strings::strings;
use tabula_data::fluent_loader::StringContext;
use ui_components::box_component::{BoxComponent, BoxComponentBuilder, Named};
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::display_state;
use crate::panels::panel_rendering;
use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};
use crate::rendering::labels;

pub fn interface_view(builder: &ResponseBuilder, battle: &BattleState) -> InterfaceView {
    let current_panel_address = display_state::get_current_panel_address(builder);
    let has_panel = current_panel_address.is_some();

    if builder.is_for_animation() {
        let overlay_builder = overlay_builder().child(
            current_panel_address
                .map(|address| panel_rendering::render_panel(address, builder, battle)),
        );

        return InterfaceView {
            has_open_panels: has_panel,
            screen_overlay: overlay_builder.build().flex_node(),
            dev_button: Some(ButtonView {
                label: strings::dev_menu_button().to_string(),
                action: None,
            }),
            undo_button: Some(ButtonView { label: strings::undo_icon().to_string(), action: None }),
            ..Default::default()
        };
    }

    let overlay_builder = overlay_builder()
        .child(render_prompt_message(builder, battle))
        .child(render_show_battlefield_button(builder, battle))
        .child(
            current_panel_address
                .map(|address| panel_rendering::render_panel(address, builder, battle)),
        );

    let overlay = overlay_builder.build().flex_node();
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());

    InterfaceView {
        has_open_panels: has_panel,
        screen_overlay: overlay,
        primary_action_button: primary_action_button(builder, battle, &legal_actions),
        primary_action_show_on_idle_duration: None,
        secondary_action_button: secondary_action_button(battle, &legal_actions),
        increment_button: increment_button(builder, battle),
        decrement_button: decrement_button(builder, battle),
        dev_button: Some(ButtonView {
            label: strings::dev_menu_button().to_string(),
            action: Some(BattleDisplayAction::OpenPanel(PanelAddress::Developer).into()),
        }),
        undo_button: Some(ButtonView {
            label: strings::undo_icon().to_string(),
            action: can_undo(builder, battle).then_some(GameAction::Undo(builder.act_for_player())),
        }),
        browser: card_browser_view(builder),
        card_order_selector: card_order_selector_view(builder, battle),
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
        None => get_generic_prompt_message(builder, &prompt.prompt_type),
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
    if matches!(prompt.prompt_type, PromptType::ModalEffect(..)) {
        // Do not apply message to modal choices.
        return None;
    }

    let prompt = match prompt.source.card_source() {
        CardSource::CardId(card_id) => {
            // TODO: Handle multiple prompts per card.
            let definition = card::get_definition(battle, card_id);
            definition.displayed_prompts.first()?.clone()
        }
        CardSource::DreamwellCard(dreamwell_card_id) => {
            let definition = battle.dreamwell.definition(dreamwell_card_id);
            definition.displayed_prompts.first()?.clone()
        }
        CardSource::None => {
            return None;
        }
    };

    battle
        .tabula
        .strings
        .format_display_string(&prompt, StringContext::Interface, FluentArgs::default())
        .ok()
}

fn get_generic_prompt_message(_builder: &ResponseBuilder, prompt_type: &PromptType) -> String {
    match prompt_type {
        PromptType::ChooseCharacter { .. } => strings::prompt_choose_character().to_string(),
        PromptType::ChooseStackCard { .. } => strings::prompt_select_stack_card().to_string(),
        PromptType::ChooseVoidCard { .. } => strings::prompt_select_from_void().to_string(),
        PromptType::ChooseHandCards { .. } => strings::prompt_select_from_hand().to_string(),
        PromptType::Choose { .. } => strings::prompt_select_option().to_string(),
        PromptType::ChooseEnergyValue { .. } => strings::prompt_choose_energy_amount().to_string(),
        PromptType::SelectDeckCardOrder { .. } => strings::prompt_select_card_order().to_string(),
        PromptType::ModalEffect(_) => strings::prompt_pick_mode().to_string(),
        PromptType::ChooseActivatedAbility { .. } => strings::prompt_select_option().to_string(),
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
                label: strings::pay_energy_additional_cost_button(current.0).to_string(),
                action: Some(BattleAction::SelectEnergyAdditionalCost(current).into()),
            });
        }
    }

    if legal_actions.contains(BattleAction::SubmitVoidCardTargets, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_submit_void_card_targets().to_string(),
            action: Some(BattleAction::SubmitVoidCardTargets.into()),
        })
    } else if legal_actions.contains(BattleAction::SubmitHandCardTargets, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_submit_hand_card_targets().to_string(),
            action: Some(BattleAction::SubmitHandCardTargets.into()),
        })
    } else if legal_actions.contains(BattleAction::SubmitDeckCardOrder, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_submit_deck_card_order().to_string(),
            action: Some(BattleAction::SubmitDeckCardOrder.into()),
        })
    } else if legal_actions.contains(BattleAction::PassPriority, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_resolve_stack().to_string(),
            action: Some(BattleAction::PassPriority.into()),
        })
    } else if legal_actions.contains(BattleAction::EndTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_end_turn().to_string(),
            action: Some(BattleAction::EndTurn.into()),
        })
    } else if legal_actions.contains(BattleAction::StartNextTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: strings::primary_button_start_next_turn().to_string(),
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
            label: strings::increment_energy_prompt_button().to_string(),
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
            label: strings::decrement_energy_prompt_button().to_string(),
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

fn render_show_battlefield_button(
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> Option<impl Component> {
    if builder.is_for_animation() {
        return None;
    }

    let has_stack = battle.cards.has_stack();
    let has_card_order_selector_prompt = battle
        .prompts
        .front()
        .map(|p| matches!(p.prompt_type, PromptType::SelectDeckCardOrder { .. }))
        .unwrap_or(false);
    let has_browser = display_state::get_card_browser_source(builder).is_some();
    let has_active_dreamwell_card = has_active_dreamwell_card(battle);
    if !(has_stack || has_card_order_selector_prompt || has_browser || has_active_dreamwell_card) {
        return None;
    }

    let label = if display_state::is_battlefield_shown(builder) {
        strings::hide_battlefield_button().to_string()
    } else {
        strings::show_battlefield_button().to_string()
    };

    Some(
        BoxComponent::builder()
            .name("Show Battlefield Button Container")
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

fn can_undo(builder: &ResponseBuilder, battle: &BattleState) -> bool {
    if builder.is_for_animation() {
        return false;
    }

    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    // Only show enabled button when some other legal action exists
    builder.provider().can_undo(battle.id, builder.display_for_player())
        && !legal_actions.is_empty()
}

fn card_browser_view(builder: &ResponseBuilder) -> Option<CardBrowserView> {
    if display_state::get_card_browser_source(builder).is_some()
        && !display_state::is_battlefield_shown(builder)
    {
        Some(CardBrowserView {
            close_button: Some(GameAction::BattleDisplayAction(
                BattleDisplayAction::CloseCardBrowser,
            )),
        })
    } else {
        None
    }
}

fn card_order_selector_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> Option<CardOrderSelectorView> {
    if display_state::is_battlefield_shown(builder) {
        return None;
    }
    if let Some(prompt) = battle.prompts.front()
        && let PromptType::SelectDeckCardOrder { .. } = &prompt.prompt_type
    {
        Some(CardOrderSelectorView { include_deck: true, include_void: true })
    } else {
        None
    }
}

fn has_active_dreamwell_card(battle: &BattleState) -> bool {
    battle.phase == BattleTurnPhase::Dreamwell
        && battle.ability_state.until_end_of_turn.active_dreamwell_card.is_some()
}
