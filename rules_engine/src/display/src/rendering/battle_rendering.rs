use std::sync::{LazyLock, Mutex};

use action_data::battle_action_data::BattleAction;
use action_data::game_action::GameAction;
use action_data::panel_address::PanelAddress;
use assert_with::{assert_that, expect, panic_with};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle_player::player_data::PlayerData;
use battle_data::prompt_types::prompt_data::Prompt;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions::LegalActions;
use battle_queries::player_queries::spark_total;
use core_data::display_color;
use core_data::numerics::Energy;
use display_data::battle_view::{BattleView, ButtonView, InterfaceView, PlayerView};
use display_data::command::{Command, GameMessageType, UpdateBattleCommand};
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify};
use masonry::flex_node::FlexNode;
use masonry::flex_style::FlexStyle;
use ui_components::component::Component;

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::panels::developer_panel::DeveloperPanel;
use crate::rendering::card_rendering;

static CURRENT_PANEL: LazyLock<Mutex<Option<FlexNode>>> = LazyLock::new(|| Mutex::new(None));

pub fn run(builder: &mut ResponseBuilder, battle: &BattleData) {
    builder.push(Command::UpdateBattle(UpdateBattleCommand {
        battle: battle_view(builder, battle),
        update_sound: None,
    }));

    if let BattleStatus::GameOver { winner } = battle.status {
        builder.push(Command::DisplayGameMessage(if winner == builder.display_for_player() {
            GameMessageType::Victory
        } else {
            GameMessageType::Defeat
        }));
    }
}

pub fn battle_view(builder: &ResponseBuilder, battle: &BattleData) -> BattleView {
    let cards = battle
        .cards
        .all_cards()
        .map(|c| card_rendering::card_view(builder, &CardViewContext::Battle(battle, c)))
        .collect::<Vec<_>>();

    BattleView {
        id: battle.id,
        user: player_view(battle, battle.player(builder.display_for_player())),
        enemy: player_view(battle, battle.player(builder.display_for_player().opponent())),
        cards,
        interface: interface_view(builder, battle),
    }
}

/// Opens a panel based on its [PanelAddress], replacing any
/// previously-displayed panel.
pub fn open_panel(address: PanelAddress) {
    let mut current_panel = CURRENT_PANEL.lock().unwrap();
    match address {
        PanelAddress::Developer => {
            *current_panel = DeveloperPanel.flex_node();
        }
    }
}

/// Closes the currently-displayed panel.
pub fn close_current_panel() {
    let mut current_panel = CURRENT_PANEL.lock().unwrap();
    *current_panel = None;
}

fn player_view(battle: &BattleData, player: &PlayerData) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: true,
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: spark_total::query(battle, player.name),
    }
}

fn interface_view(builder: &ResponseBuilder, battle: &BattleData) -> InterfaceView {
    if builder.is_for_animation() {
        return InterfaceView::default();
    }

    let current_panel = CURRENT_PANEL.lock().unwrap().clone();
    let legal_actions = legal_actions::compute(battle, builder.act_for_player(), LegalActions {
        for_human_player: true,
    });

    InterfaceView {
        screen_overlay: current_panel,
        primary_action_button: primary_action_button(builder, battle, &legal_actions),
        primary_action_show_on_idle_duration: None,
        secondary_action_button: secondary_action_button(battle, &legal_actions),
        increment_button: increment_button(builder, battle),
        decrement_button: decrement_button(builder, battle),
        dev_button: Some(ButtonView {
            label: "\u{f0ad} Dev".to_string(),
            action: Some(GameAction::OpenPanel(PanelAddress::Developer)),
        }),
        card_order_selector: None,
        bottom_right_button: None,
    }
}

fn primary_action_button(
    builder: &ResponseBuilder,
    battle: &BattleData,
    legal_actions: &[BattleAction],
) -> Option<ButtonView> {
    if legal_actions.contains(&BattleAction::SelectPromptChoice(0)) {
        let prompt = expect!(battle.prompt.as_ref(), battle, || {
            "Expected prompt for SelectPromptChoice action"
        });
        let Prompt::Choose { choices } = &prompt.prompt else {
            panic_with!(battle, "Expected a Choose prompt");
        };
        assert_that!(!choices.is_empty(), battle, || "Expected a Choose prompt with choices");
        return Some(ButtonView {
            label: choices[0].label.clone(),
            action: Some(BattleAction::SelectPromptChoice(0).into()),
        });
    }

    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let Prompt::ChooseEnergyValue { current, .. } = &prompt.prompt
    {
        return Some(ButtonView {
            label: format!("Spend {}\u{f7e4}", current),
            action: Some(BattleAction::SelectEnergyAdditionalCost(*current).into()),
        });
    }

    if legal_actions.contains(&BattleAction::PassPriority) {
        Some(ButtonView {
            label: "Resolve".to_string(),
            action: Some(BattleAction::PassPriority.into()),
        })
    } else if legal_actions.contains(&BattleAction::EndTurn) {
        Some(ButtonView {
            label: "End Turn".to_string(),
            action: Some(BattleAction::EndTurn.into()),
        })
    } else if legal_actions.contains(&BattleAction::StartNextTurn) {
        Some(ButtonView {
            label: "Next Turn".to_string(),
            action: Some(BattleAction::StartNextTurn.into()),
        })
    } else {
        None
    }
}

fn secondary_action_button(
    battle: &BattleData,
    legal_actions: &[BattleAction],
) -> Option<ButtonView> {
    if legal_actions.contains(&BattleAction::SelectPromptChoice(1)) {
        let prompt = expect!(battle.prompt.as_ref(), battle, || {
            "Expected prompt for SelectPromptChoice action"
        });
        let Prompt::Choose { choices } = &prompt.prompt else {
            panic_with!(battle, "Expected a Choose prompt");
        };
        assert_that!(!choices.is_empty(), battle, || "Expected a Choose prompt with choices");
        Some(ButtonView {
            label: choices[1].label.clone(),
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

fn increment_button(builder: &ResponseBuilder, battle: &BattleData) -> Option<ButtonView> {
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let Prompt::ChooseEnergyValue { current, .. } = &prompt.prompt
    {
        return Some(ButtonView {
            label: "+1\u{f7e4}".to_string(),
            action: if *current < battle.player(builder.act_for_player()).current_energy {
                Some(BattleAction::SetSelectedEnergyAdditionalCost(*current + Energy(1)).into())
            } else {
                None
            },
        });
    }

    None
}

fn decrement_button(builder: &ResponseBuilder, battle: &BattleData) -> Option<ButtonView> {
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.act_for_player()
        && let Prompt::ChooseEnergyValue { current, .. } = &prompt.prompt
    {
        return Some(ButtonView {
            label: "\u{2212}1\u{f7e4}".to_string(),
            action: if *current > Energy(0) {
                Some(BattleAction::SetSelectedEnergyAdditionalCost(*current - Energy(1)).into())
            } else {
                None
            },
        });
    }

    None
}
