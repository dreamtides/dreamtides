use battle_queries::battle_card_queries::stack_card_queries;
use battle_queries::battle_player_queries::player_properties;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::battle_player::battle_player_state::BattlePlayerState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;
use display_data::battle_view::{BattlePreviewState, BattleView, PlayerView};
use display_data::command::{ArrowStyle, Command, DisplayArrow, GameMessageType};

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::{display_state, outcome_simulation};
use crate::rendering::{card_rendering, identity_card_rendering, interface_rendering};

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
    let mut cards = battle
        .cards
        .all_cards()
        .map(|id| {
            card_rendering::card_view(
                builder,
                &CardViewContext::Battle(battle, battle.cards.card(id).name, id),
            )
        })
        .collect::<Vec<_>>();

    cards.push(identity_card_rendering::identity_card_view(
        builder,
        battle,
        builder.display_for_player(),
    ));
    cards.push(identity_card_rendering::identity_card_view(
        builder,
        battle,
        builder.display_for_player().opponent(),
    ));

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
        interface: interface_rendering::interface_view(builder, battle),
        arrows: current_arrows(builder, battle),
        preview: if builder.is_for_animation() {
            BattlePreviewState::Pending
        } else {
            outcome_simulation::current_prompt_battle_preview(battle, builder.display_for_player())
                .map(BattlePreviewState::Active)
                .unwrap_or(BattlePreviewState::None)
        },
    }
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
            stack_card_queries::targets(battle, stack_card.id).map(|targets| {
                let source = adapter::card_game_object_id(stack_card.id);
                let (target, color) = match targets {
                    StackCardTargets::Character(character_id, _) => {
                        (adapter::card_game_object_id(*character_id), ArrowStyle::Red)
                    }
                    StackCardTargets::StackCard(stack_card_id, _) => {
                        (adapter::card_game_object_id(*stack_card_id), ArrowStyle::Blue)
                    }
                };
                DisplayArrow { source, target, color }
            })
        })
        .collect()
}
