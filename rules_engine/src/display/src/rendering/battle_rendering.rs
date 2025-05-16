use std::sync::{LazyLock, Mutex};

use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::battle_player_queries::player_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle_player::battle_player_state::BattlePlayerState;
use battle_state::prompt_types::prompt_data::{PromptChoiceLabel, PromptType};
use core_data::display_color;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use display_data::battle_view::{BattleView, ButtonView, InterfaceView, PlayerView};
use display_data::command::{Command, GameMessageType, UpdateBattleCommand};
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify};
use masonry::flex_style::FlexStyle;
use tracing_macros::panic_with;
use ui_components::icon;

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::panels::panel_rendering;
use crate::rendering::card_rendering;

static CURRENT_PANEL_ADDRESS: LazyLock<Mutex<Option<PanelAddress>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn run(builder: &mut ResponseBuilder, battle: &BattleState) {
    builder.push(Command::UpdateBattle(UpdateBattleCommand {
        battle: battle_view(builder, battle),
        update_sound: None,
    }));

    if let BattleStatus::GameOver { winner } = battle.status {
        builder.push(Command::DisplayGameMessage(
            if winner == Some(builder.display_for_player()) {
                GameMessageType::Victory
            } else {
                GameMessageType::Defeat
            },
        ));
    }
}

pub fn battle_view(builder: &ResponseBuilder, battle: &BattleState) -> BattleView {
    let cards = battle
        .cards
        .all_cards()
        .map(|id| {
            card_rendering::card_view(
                builder,
                &CardViewContext::Battle(battle, battle.cards.name(id), id),
            )
        })
        .collect::<Vec<_>>();

    BattleView {
        id: battle.id,
        user: player_view(
            battle,
            builder.display_for_player(),
            battle.players.player(builder.display_for_player()),
        ),
        enemy: player_view(
            battle,
            builder.display_for_player().opponent(),
            battle.players.player(builder.display_for_player().opponent()),
        ),
        cards,
        interface: interface_view(builder, battle),
    }
}

/// Opens a panel based on its [PanelAddress], replacing any
/// previously-displayed panel.
pub fn open_panel(address: PanelAddress) {
    let mut current_panel_address = CURRENT_PANEL_ADDRESS.lock().unwrap();
    *current_panel_address = Some(address);
}

/// Closes the currently-displayed panel.
pub fn close_current_panel() {
    let mut current_panel_address = CURRENT_PANEL_ADDRESS.lock().unwrap();
    *current_panel_address = None;
}

fn player_view(battle: &BattleState, name: PlayerName, player: &BattlePlayerState) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: true,
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: player_properties::spark_total(battle, name),
    }
}

fn interface_view(builder: &ResponseBuilder, battle: &BattleState) -> InterfaceView {
    if builder.is_for_animation() {
        return InterfaceView::default();
    }

    let current_panel_address = *CURRENT_PANEL_ADDRESS.lock().unwrap();
    let screen_overlay = current_panel_address
        .and_then(|address| panel_rendering::render_panel(address, builder, battle));
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());

    InterfaceView {
        screen_overlay,
        primary_action_button: primary_action_button(builder, battle, &legal_actions),
        primary_action_show_on_idle_duration: None,
        secondary_action_button: secondary_action_button(battle, &legal_actions),
        increment_button: increment_button(builder, battle),
        decrement_button: decrement_button(builder, battle),
        dev_button: Some(ButtonView {
            label: "\u{f0ad} Dev".to_string(),
            action: Some(GameAction::OpenPanel(PanelAddress::Developer)),
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
            label: prompt_choice_label(choices[0].label),
            action: Some(BattleAction::SelectPromptChoice(0).into()),
        });
    }

    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { current, .. } = &prompt.prompt_type
        && legal_actions.contains(BattleAction::SelectEnergyAdditionalCost(*current))
    {
        return Some(ButtonView {
            label: format!("Spend {}\u{f7e4}", current),
            action: Some(BattleAction::SelectEnergyAdditionalCost(*current).into()),
        });
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
            label: prompt_choice_label(choices[1].label),
            action: Some(BattleAction::SelectPromptChoice(1).into()),
        })
    } else {
        None
    }
}

pub fn create_flex_style() -> FlexStyle {
    FlexStyle::builder()
        .padding((8, 12))
        .margin(4)
        .flex_direction(FlexDirection::Row)
        .flex_basis(12)
        .flex_grow(1)
        .flex_shrink(1)
        .align_items(FlexAlign::Center)
        .justify_content(FlexJustify::Center)
        .border_radius(4)
        .border_width(1)
        .border_color(display_color::RED_100)
        .build()
}

fn increment_button(builder: &ResponseBuilder, battle: &BattleState) -> Option<ButtonView> {
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let PromptType::ChooseEnergyValue { current, maximum, .. } = &prompt.prompt_type
    {
        return Some(ButtonView {
            label: "+1\u{f7e4}".to_string(),
            action: if *current + Energy(1) <= *maximum {
                Some(BattleAction::SetSelectedEnergyAdditionalCost(*current + Energy(1)).into())
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
        && let PromptType::ChooseEnergyValue { current, minimum, .. } = &prompt.prompt_type
    {
        return Some(ButtonView {
            label: "-1\u{f7e4}".to_string(),
            action: if *current > Energy(0) && *current - Energy(1) >= *minimum {
                Some(BattleAction::SetSelectedEnergyAdditionalCost(*current - Energy(1)).into())
            } else {
                None
            },
        });
    }

    None
}

fn prompt_choice_label(label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::PayEnergy(energy) => format!("Spend {}{}", energy, icon::ENERGY),
        PromptChoiceLabel::Decline => "Decline".to_string(),
    }
}
