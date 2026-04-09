use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    LegalActions, StandardLegalActions,
};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use core_data::types::PlayerName;
use serde::Serialize;

use crate::position_assignment;

#[derive(Clone, Serialize)]
pub struct EvaluationBreakdown {
    pub total: f64,
    pub features: Vec<EvaluationFeature>,
}

#[derive(Clone, Serialize)]
pub struct EvaluationFeature {
    pub name: String,
    pub value: f64,
}

pub fn score(battle: &BattleState, maximizing_player: PlayerName) -> f64 {
    breakdown(battle, maximizing_player).total
}

pub fn breakdown(battle: &BattleState, maximizing_player: PlayerName) -> EvaluationBreakdown {
    let mut features = Vec::new();

    if let BattleStatus::GameOver { winner } = battle.status {
        let total = if winner == Some(maximizing_player) {
            100_000.0
        } else if winner == Some(maximizing_player.opponent()) {
            -100_000.0
        } else {
            0.0
        };
        features.push(EvaluationFeature { name: "terminal".to_string(), value: total });
        return EvaluationBreakdown { total, features };
    }

    push_feature(&mut features, "points_margin", points_margin_score(battle, maximizing_player));
    push_feature(
        &mut features,
        "distance_to_win",
        distance_to_win_score(battle, maximizing_player),
    );
    push_feature(&mut features, "front_pressure", front_pressure_score(battle, maximizing_player));
    push_feature(&mut features, "board_presence", board_presence_score(battle, maximizing_player));
    push_feature(
        &mut features,
        "blocker_coverage",
        blocker_coverage_score(battle, maximizing_player),
    );
    push_feature(&mut features, "resource_flow", resource_flow_score(battle, maximizing_player));
    push_feature(&mut features, "hand_quality", hand_quality_score(battle, maximizing_player));
    push_feature(&mut features, "void_quality", void_quality_score(battle, maximizing_player));
    push_feature(&mut features, "stack_pressure", stack_pressure_score(battle, maximizing_player));
    push_feature(
        &mut features,
        "positioning_value",
        positioning_value_score(battle, maximizing_player),
    );

    EvaluationBreakdown { total: features.iter().map(|feature| feature.value).sum(), features }
}

fn push_feature(features: &mut Vec<EvaluationFeature>, name: &str, value: f64) {
    features.push(EvaluationFeature { name: name.to_string(), value });
}

fn points_margin_score(battle: &BattleState, player: PlayerName) -> f64 {
    let our_points = f64::from(battle.players.player(player).points.0);
    let their_points = f64::from(battle.players.player(player.opponent()).points.0);
    (our_points - their_points) * 140.0
}

fn distance_to_win_score(battle: &BattleState, player: PlayerName) -> f64 {
    let points_to_win = i64::from(battle.rules_config.points_to_win.0);
    let our_remaining = points_to_win - i64::from(battle.players.player(player).points.0);
    let their_remaining =
        points_to_win - i64::from(battle.players.player(player.opponent()).points.0);
    f64::from((their_remaining - our_remaining) as i32) * 45.0
}

fn front_pressure_score(battle: &BattleState, player: PlayerName) -> f64 {
    let mut total = 0.0;
    let our_front = &battle.cards.battlefield(player).front;
    let their_front = &battle.cards.battlefield(player.opponent()).front;

    for column in 0..battle.rules_config.front_row_size {
        let our_character = our_front[column];
        let their_character = their_front[column];
        total += column_pressure_score(battle, player, our_character, their_character);
        total -= column_pressure_score(battle, player.opponent(), their_character, our_character);
    }

    if battle.phase == BattleTurnPhase::Judgment && battle.turn.active_player == player {
        total *= 1.25;
    }

    total
}

fn column_pressure_score(
    battle: &BattleState,
    controller: PlayerName,
    attacker: Option<CharacterId>,
    blocker: Option<CharacterId>,
) -> f64 {
    match (attacker, blocker) {
        (Some(attacker_id), Some(blocker_id)) => {
            let attacker_spark = f64::from(
                card_properties::spark(battle, controller, attacker_id).unwrap_or_default().0,
            );
            let blocker_spark = f64::from(
                card_properties::spark(battle, controller.opponent(), blocker_id)
                    .unwrap_or_default()
                    .0,
            );
            let trade_value = attacker_spark - blocker_spark;
            trade_value * 22.0
                + if attacker_spark > blocker_spark {
                    blocker_spark * 18.0
                } else if blocker_spark > attacker_spark {
                    -attacker_spark * 20.0
                } else {
                    -(attacker_spark + blocker_spark) * 10.0
                }
        }
        (Some(attacker_id), None) => {
            let spark = f64::from(
                card_properties::spark(battle, controller, attacker_id).unwrap_or_default().0,
            );
            spark * 55.0
        }
        _ => 0.0,
    }
}

fn board_presence_score(battle: &BattleState, player: PlayerName) -> f64 {
    side_board_presence_score(battle, player) - side_board_presence_score(battle, player.opponent())
}

fn side_board_presence_score(battle: &BattleState, player: PlayerName) -> f64 {
    let current_turn = battle.turn.turn_id.0;
    let moved_this_turn = &battle.turn.moved_this_turn;
    battle
        .cards
        .battlefield(player)
        .all_characters()
        .into_iter()
        .map(|character_id| {
            let spark = f64::from(
                card_properties::spark(battle, player, character_id).unwrap_or_default().0,
            );
            let is_back = battle.cards.battlefield(player).is_in_back_rank(character_id);
            let state = battle.cards.battlefield_state(player).get(&character_id);
            let ready = state
                .is_some_and(|battlefield_state| battlefield_state.played_turn != current_turn)
                && !moved_this_turn.contains(&character_id);
            if is_back { if ready { spark * 18.0 } else { spark * 10.0 } } else { spark * 30.0 }
        })
        .sum()
}

fn blocker_coverage_score(battle: &BattleState, player: PlayerName) -> f64 {
    coverage_for_player(battle, player) - coverage_for_player(battle, player.opponent())
}

fn coverage_for_player(battle: &BattleState, player: PlayerName) -> f64 {
    let threats = opponent_front_threats(battle, player);
    let blockers = available_back_blockers(battle, player);
    threats
        .iter()
        .map(|threat| {
            let winning = blockers.iter().any(|spark| *spark >= threat.1);
            let chump = blockers.iter().any(|spark| *spark > 0.0);
            if winning {
                35.0 + threat.1 * 10.0
            } else if chump {
                10.0 + threat.1 * 4.0
            } else {
                -threat.1 * 8.0
            }
        })
        .sum()
}

fn opponent_front_threats(battle: &BattleState, player: PlayerName) -> Vec<(usize, f64)> {
    battle.cards.battlefield(player.opponent()).front[..battle.rules_config.front_row_size]
        .iter()
        .enumerate()
        .filter_map(|(column, slot)| {
            slot.map(|character_id| {
                let spark = f64::from(
                    card_properties::spark(battle, player.opponent(), character_id)
                        .unwrap_or_default()
                        .0,
                );
                (column, spark)
            })
        })
        .collect()
}

fn available_back_blockers(battle: &BattleState, player: PlayerName) -> Vec<f64> {
    let current_turn = battle.turn.turn_id.0;
    battle.cards.battlefield(player).back[..battle.rules_config.back_row_size]
        .iter()
        .flatten()
        .filter(|character_id| {
            battle
                .cards
                .battlefield_state(player)
                .get(character_id)
                .is_some_and(|state| state.played_turn != current_turn)
                && !battle.turn.moved_this_turn.contains(character_id)
        })
        .map(|character_id| {
            f64::from(card_properties::spark(battle, player, *character_id).unwrap_or_default().0)
        })
        .collect()
}

fn resource_flow_score(battle: &BattleState, player: PlayerName) -> f64 {
    let our_player = battle.players.player(player);
    let their_player = battle.players.player(player.opponent());
    let energy_margin =
        f64::from(our_player.current_energy.0) - f64::from(their_player.current_energy.0);
    let production_margin =
        f64::from(our_player.produced_energy.0) - f64::from(their_player.produced_energy.0);
    energy_margin * 5.0 + production_margin * 9.0
}

fn hand_quality_score(battle: &BattleState, player: PlayerName) -> f64 {
    zone_value_for_hand(battle, player) - zone_value_for_hand(battle, player.opponent())
}

fn zone_value_for_hand(battle: &BattleState, player: PlayerName) -> f64 {
    f64::from(battle.cards.hand(player).len() as u32) * 12.0
        + battle
            .cards
            .hand(player)
            .iter()
            .map(|card_id| card_zone_value(battle, player, card_id))
            .sum::<f64>()
}

fn void_quality_score(battle: &BattleState, player: PlayerName) -> f64 {
    zone_value_for_void(battle, player) - zone_value_for_void(battle, player.opponent())
}

fn zone_value_for_void(battle: &BattleState, player: PlayerName) -> f64 {
    battle
        .cards
        .void(player)
        .iter()
        .map(|card_id| card_zone_value(battle, player, card_id) * 0.5)
        .sum()
}

fn card_zone_value(battle: &BattleState, controller: PlayerName, card_id: impl CardIdType) -> f64 {
    let energy_cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
    let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
    let fast_bonus = if card_properties::is_fast(battle, card_id) { 6.0 } else { 0.0 };
    let owner_bonus =
        if card_properties::controller(battle, card_id) == controller { 0.0 } else { -5.0 };
    energy_cost * 1.8 + spark * 2.4 + fast_bonus + owner_bonus
}

fn stack_pressure_score(battle: &BattleState, player: PlayerName) -> f64 {
    if !battle.cards.has_stack() {
        return 0.0;
    }

    let mut total = if battle.stack_priority == Some(player) { 30.0 } else { -30.0 };

    if let Some(top) = battle.cards.top_of_stack() {
        let top_controller = top.controller;
        total += if top_controller == player { 18.0 } else { -18.0 };
        if let Some(targets) = &top.targets {
            let targeted_player = targets
                .card_ids()
                .into_iter()
                .filter_map(|card_id| {
                    let character_id = CharacterId(card_id);
                    if battle.cards.battlefield(player).contains(character_id) {
                        Some(player)
                    } else if battle.cards.battlefield(player.opponent()).contains(character_id) {
                        Some(player.opponent())
                    } else {
                        None
                    }
                })
                .next();
            if targeted_player == Some(player) {
                total -= 20.0;
            } else if targeted_player == Some(player.opponent()) {
                total += 20.0;
            }
        }
    }

    total
}

fn positioning_value_score(battle: &BattleState, player: PlayerName) -> f64 {
    let legal = legal_actions::compute(battle, player);
    let can_position = matches!(legal, LegalActions::Standard {
        actions: StandardLegalActions { can_begin_positioning: true, .. },
    });
    let our_best = if can_position {
        position_assignment::best_assignment(battle, player)
            .map(|assignment| position_assignment::describe(battle, player, &assignment))
            .map_or(0.0, |description| assignment_description_score(&description))
    } else {
        0.0
    };
    let their_best = if battle.turn.active_player == player.opponent() {
        position_assignment::best_assignment(battle, player.opponent())
            .map(|assignment| position_assignment::describe(battle, player.opponent(), &assignment))
            .map_or(0.0, |description| assignment_description_score(&description))
    } else {
        0.0
    };
    our_best - their_best
}

fn assignment_description_score(description: &str) -> f64 {
    if description == "hold-all" {
        8.0
    } else {
        let attack_count = description.matches("attack-").count() as f64;
        let block_count = description.matches("block-").count() as f64;
        let chump_count = description.matches("chump-").count() as f64;
        attack_count * 12.0 + block_count * 18.0 + chump_count * 10.0
    }
}
