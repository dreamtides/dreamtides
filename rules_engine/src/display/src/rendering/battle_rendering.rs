use std::sync::{LazyLock, Mutex};

use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_queries::battle_player_queries::player_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::battle_player::battle_player_state::BattlePlayerState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use display_data::battle_view::{
    BattlePreviewView, BattleView, ButtonView, InterfaceView, PlayerPreviewView, PlayerView,
};
use display_data::command::{ArrowStyle, Command, DisplayArrow, GameMessageType, GameObjectId};
use masonry::flex_node::FlexNode;
use tracing_macros::panic_with;
use ui_components::component::Component;

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::{display_state, outcome_simulation};
use crate::panels::panel_rendering;
use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};
use crate::rendering::{card_rendering, labels};

static CURRENT_PANEL_ADDRESS: LazyLock<Mutex<Option<PanelAddress>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn run(builder: &mut ResponseBuilder, battle: &BattleState) {
    builder.push_battle_view(battle_view(builder, battle));
    update_display_state(battle);

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
                &CardViewContext::Battle(battle, battle.cards.card(id).name, id),
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
        arrows: current_arrows(builder, battle),
        preview: battle_preview(builder, battle),
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

fn update_display_state(battle: &BattleState) {
    // Clear energy selection when no energy prompt is active
    if battle
        .prompt
        .as_ref()
        .map(|p| &p.prompt_type)
        .is_none_or(|pt| !matches!(pt, PromptType::ChooseEnergyValue { .. }))
    {
        display_state::clear_selected_energy_additional_cost();
    }
}

fn player_view(battle: &BattleState, name: PlayerName, player: &BattlePlayerState) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: true,
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: player_properties::spark_total(battle, name),
        is_current_turn: battle.turn.active_player == name,
        is_victory_imminent: outcome_simulation::is_victory_imminent_for_player(battle, name),
    }
}

fn current_arrows(builder: &ResponseBuilder, battle: &BattleState) -> Vec<DisplayArrow> {
    if builder.is_for_animation() {
        return vec![];
    }

    battle
        .cards
        .all_cards_on_stack()
        .iter()
        .filter_map(|stack_card| {
            stack_card.targets.as_ref().map(|targets| {
                let source = GameObjectId::CardId(stack_card.id.card_id());
                let (target, color) = match targets {
                    StackCardTargets::Character(character_id) => {
                        (GameObjectId::CardId(character_id.card_id()), ArrowStyle::Red)
                    }
                    StackCardTargets::StackCard(stack_card_id) => {
                        (GameObjectId::CardId(stack_card_id.card_id()), ArrowStyle::Blue)
                    }
                };
                DisplayArrow { source, target, color }
            })
        })
        .collect()
}

fn interface_view(builder: &ResponseBuilder, battle: &BattleState) -> InterfaceView {
    if builder.is_for_animation() {
        return InterfaceView::default();
    }

    let current_panel_address = *CURRENT_PANEL_ADDRESS.lock().unwrap();
    let panel = current_panel_address
        .and_then(|address| panel_rendering::render_panel(address, builder, battle));
    let screen_overlay = screen_overlay_stack(vec![interface_message(), panel]);
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

fn interface_message() -> Option<FlexNode> {
    InterfaceMessage::builder()
        .text("Hello, world")
        .anchor_position(AnchorPosition::Top)
        .build()
        .flex_node()
}

fn screen_overlay_stack(items: Vec<Option<FlexNode>>) -> Option<FlexNode> {
    None
}

fn battle_preview(builder: &ResponseBuilder, battle: &BattleState) -> Option<BattlePreviewView> {
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == builder.display_for_player()
        && let PromptType::ChooseEnergyValue { minimum, .. } = &prompt.prompt_type
    {
        let current = display_state::get_selected_energy_additional_cost().unwrap_or(*minimum);
        let player = battle.players.player(builder.display_for_player());
        let remaining_energy = player.current_energy - current;

        Some(BattlePreviewView {
            user: PlayerPreviewView { energy: Some(remaining_energy), ..Default::default() },
            enemy: PlayerPreviewView::default(),
            cards: vec![],
            preview_message: None,
        })
    } else {
        None
    }
}
