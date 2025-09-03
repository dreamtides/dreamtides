use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::CardSource;
use battle_state::prompt_types::prompt_data::{PromptData, PromptType};
use core_data::numerics::Energy;
use display_data::battle_view::{
    ButtonView, CardBrowserView, CardOrderSelectorView, InterfaceView,
};
use fluent::{FluentArgs, fluent_args};
use masonry::dimension::{FlexInsets, SafeAreaInsets};
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;
use tabula_data::localized_strings::StringContext;
use tabula_ids::string_id;
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
                label: builder.string(string_id::DEV_MENU_BUTTON),
                action: None,
            }),
            undo_button: Some(ButtonView {
                label: builder.string(string_id::UNDO_ICON),
                action: None,
            }),
            ..Default::default()
        };
    }

    let overlay_builder = overlay_builder()
        .child(render_prompt_message(builder, battle))
        .child(render_hide_overlay_button(builder, battle))
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
        secondary_action_button: secondary_action_button(builder, battle, &legal_actions),
        increment_button: increment_button(builder, battle),
        decrement_button: decrement_button(builder, battle),
        dev_button: Some(ButtonView {
            label: builder.string(string_id::DEV_MENU_BUTTON),
            action: Some(BattleDisplayAction::OpenPanel(PanelAddress::Developer).into()),
        }),
        undo_button: Some(ButtonView {
            label: builder.string(string_id::UNDO_ICON),
            action: can_undo(builder, battle).then_some(GameAction::Undo(builder.act_for_player())),
        }),
        browser: card_browser_view(builder),
        card_order_selector: card_order_selector_view(builder, battle),
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

    Some(battle.tabula.strings.format_display_string(
        &prompt,
        StringContext::Interface,
        FluentArgs::default(),
    ))
}

fn get_generic_prompt_message(builder: &ResponseBuilder, prompt_type: &PromptType) -> String {
    match prompt_type {
        PromptType::ChooseCharacter { .. } => builder.string(string_id::PROMPT_CHOOSE_CHARACTER),
        PromptType::ChooseStackCard { .. } => builder.string(string_id::PROMPT_SELECT_STACK_CARD),
        PromptType::ChooseVoidCard { .. } => builder.string(string_id::PROMPT_SELECT_FROM_VOID),
        PromptType::ChooseHandCards { .. } => builder.string(string_id::PROMPT_SELECT_FROM_HAND),
        PromptType::Choose { .. } => builder.string(string_id::PROMPT_SELECT_OPTION),
        PromptType::ChooseEnergyValue { .. } => {
            builder.string(string_id::PROMPT_CHOOSE_ENERGY_AMOUNT)
        }
        PromptType::SelectDeckCardOrder { .. } => {
            builder.string(string_id::PROMPT_SELECT_CARD_ORDER)
        }
        PromptType::ModalEffect(_) => builder.string(string_id::PROMPT_PICK_MODE),
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
            label: labels::choice_label(builder, choices[0].label),
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
                label: builder.string_with_args(
                    string_id::PAY_ENERGY_ADDTIONAL_COST_BUTTON,
                    fluent_args!("energy" => current),
                ),
                action: Some(BattleAction::SelectEnergyAdditionalCost(current).into()),
            });
        }
    }

    if legal_actions.contains(BattleAction::SubmitVoidCardTargets, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_SUBMIT_VOID_CARD_TARGETS),
            action: Some(BattleAction::SubmitVoidCardTargets.into()),
        })
    } else if legal_actions.contains(BattleAction::SubmitHandCardTargets, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_SUBMIT_HAND_CARD_TARGETS),
            action: Some(BattleAction::SubmitHandCardTargets.into()),
        })
    } else if legal_actions.contains(BattleAction::SubmitDeckCardOrder, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_SUBMIT_DECK_CARD_ORDER),
            action: Some(BattleAction::SubmitDeckCardOrder.into()),
        })
    } else if legal_actions.contains(BattleAction::PassPriority, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_RESOLVE_STACK),
            action: Some(BattleAction::PassPriority.into()),
        })
    } else if legal_actions.contains(BattleAction::EndTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_END_TURN),
            action: Some(BattleAction::EndTurn.into()),
        })
    } else if legal_actions.contains(BattleAction::StartNextTurn, ForPlayer::Human) {
        Some(ButtonView {
            label: builder.string(string_id::PRIMARY_BUTTON_START_NEXT_TURN),
            action: Some(BattleAction::StartNextTurn.into()),
        })
    } else {
        None
    }
}

fn secondary_action_button(
    builder: &ResponseBuilder,
    battle: &BattleState,
    legal_actions: &LegalActions,
) -> Option<ButtonView> {
    if legal_actions.contains(BattleAction::SelectPromptChoice(1), ForPlayer::Human)
        && let Some(PromptType::Choose { choices }) = battle.prompts.front().map(|p| &p.prompt_type)
        && choices.len() > 1
    {
        Some(ButtonView {
            label: labels::choice_label(builder, choices[1].label),
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
            label: builder.string(string_id::INCREMENT_ENERGY_PROMPT_BUTTON),
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
            label: builder.string(string_id::DECREMENT_ENERGY_PROMPT_BUTTON),
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

fn render_hide_overlay_button(
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
    if !(has_stack || has_card_order_selector_prompt || has_browser) {
        return None;
    }

    let label = if display_state::is_overlay_hidden(builder) {
        builder.string(string_id::SHOW_STACK_BUTTON)
    } else {
        builder.string(string_id::HIDE_STACK_BUTTON)
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
        && !display_state::is_overlay_hidden(builder)
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
    if display_state::is_overlay_hidden(builder) {
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
