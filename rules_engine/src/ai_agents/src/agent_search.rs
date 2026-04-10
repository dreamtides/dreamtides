use std::cell::RefCell;
use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};

use ai_data::game_ai::GameAI;
use ai_strategic::{search, search_v2, search_v3};
use ai_uct::position_assignment::{CharacterPlacement, PositionAssignment};
use ai_uct::uct_config::UctConfig;
use ai_uct::{
    uct_search, uct_search_hybrid_v1, uct_search_v2, uct_search_v3, uct_search_v4, uct_search_v5,
    uct_search_v6, uct_search_v7, uct_search_v8,
};
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use rand::Rng;
use rand::seq::IndexedRandom;
use tracing::{debug, instrument};

thread_local! {
    static PENDING_ASSIGNMENT: RefCell<Option<PositionAssignment>> = const { RefCell::new(None) };
    static PENDING_STRATEGIC_ACTIONS: RefCell<VecDeque<BattleAction>> = const { RefCell::new(VecDeque::new()) };
}

/// Selects an action using a custom UctConfig (exposed for benchmarks to allow
/// forcing iteration multipliers like setting iteration_multiplier_override).
pub fn select_action_with_uct_config(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    let battle = &player_state::randomize_battle_player(
        initial_battle,
        player.opponent(),
        rand::rng().random(),
    );
    uct_search::search(battle, player, config)
}

/// Selects an action for the given player using the given AI agent.
#[instrument(skip_all, level = "debug")]
pub fn select_action(battle: &BattleState, player: PlayerName, game_ai: &GameAI) -> BattleAction {
    assert_eq!(legal_actions::next_to_act(battle), Some(player));

    let legal_actions = legal_actions::compute(battle, player);
    if legal_actions.is_empty() {
        panic_with!("No legal actions available for player", battle, player, legal_actions);
    }

    if legal_actions.len() == 1 {
        debug!("Automatically selecting action {:?}", legal_actions.all()[0]);
        return legal_actions.all()[0];
    }

    let start_time = Instant::now();
    let action = select_action_unchecked(battle, player, game_ai, None);
    debug!(
        "Agent selected action {:?} in {:.3} seconds",
        action,
        start_time.elapsed().as_secs_f64()
    );
    action
}

/// Selects an action for the given player using the given AI agent, without
/// checking for validity.
///
/// Mostly intended for use in benchmarking agents.
pub fn select_action_unchecked(
    initial_battle: &BattleState,
    player: PlayerName,
    game_ai: &GameAI,
    iteration_multiplier_override: Option<f64>,
) -> BattleAction {
    match game_ai {
        GameAI::AlwaysPanic => panic!("Always panic agent called for an action"),
        GameAI::FirstAvailableAction => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            first_available_action(battle, player)
        }
        GameAI::RandomAction => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            random_action(battle, player)
        }
        GameAI::MonteCarlo(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search::search(battle, player, &config)
        }
        GameAI::MonteCarloSingleThreaded(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: true,
            };
            uct_search::search(battle, player, &config)
        }
        GameAI::MonteCarloV2(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            if let Some(action) = next_assignment_action(battle, player) {
                return action;
            }

            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            let result = uct_search_v2::search(battle, player, &config);
            if let Some(assignment) = result.assignment {
                PENDING_ASSIGNMENT.with(|cell| {
                    *cell.borrow_mut() = Some(assignment);
                });
            }
            result.action
        }
        GameAI::MonteCarloV3(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            if let Some(action) = next_assignment_action(battle, player) {
                return action;
            }

            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            let result = uct_search_v3::search(battle, player, &config);
            if let Some(assignment) = result.assignment {
                PENDING_ASSIGNMENT.with(|cell| {
                    *cell.borrow_mut() = Some(assignment);
                });
            }
            result.action
        }
        GameAI::MonteCarloV4(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search_v4::search(battle, player, &config)
        }
        GameAI::MonteCarloV5(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search_v5::search(battle, player, &config)
        }
        GameAI::MonteCarloV6(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search_v6::search(battle, player, &config)
        }
        GameAI::MonteCarloV7(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search_v7::search(battle, player, &config)
        }
        GameAI::MonteCarloV8(thousands_of_iterations) => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search_v8::search(battle, player, &config)
        }
        GameAI::MonteCarloHybridV1(budget_ms) => {
            if !uct_search_hybrid_v1::is_available() {
                debug!(
                    "MonteCarloHybridV1 artifacts unavailable, falling back to MonteCarloV4(50)"
                );
                let battle = &player_state::randomize_battle_player(
                    initial_battle,
                    player.opponent(),
                    rand::rng().random(),
                );
                let config = UctConfig {
                    max_iterations_per_action: 50_000,
                    max_total_actions_multiplier: 6,
                    iteration_multiplier_override,
                    single_threaded: false,
                };
                return uct_search_v4::search(battle, player, &config);
            }

            uct_search_hybrid_v1::search(initial_battle, player, *budget_ms)
        }
        GameAI::StrategicV1(budget_ms) => {
            if let Some(action) = next_strategic_action(initial_battle, player) {
                return action;
            }

            let result = search::search(initial_battle, player, *budget_ms);
            validated_strategic_action(
                initial_battle,
                player,
                result.action,
                result.pending_actions,
            )
        }
        GameAI::StrategicV2(budget_ms) => {
            if let Some(action) = next_strategic_action(initial_battle, player) {
                return action;
            }

            let result = search_v2::search(initial_battle, player, *budget_ms);
            validated_strategic_action(
                initial_battle,
                player,
                result.action,
                result.pending_actions,
            )
        }
        GameAI::StrategicV3(budget_ms) => {
            if let Some(action) = next_strategic_action(initial_battle, player) {
                return action;
            }

            if !search_v3::is_available() {
                debug!("StrategicV3 artifacts unavailable, falling back to MonteCarlo(50)");
                let battle = &player_state::randomize_battle_player(
                    initial_battle,
                    player.opponent(),
                    rand::rng().random(),
                );
                let config = UctConfig {
                    max_iterations_per_action: 50_000,
                    max_total_actions_multiplier: 6,
                    iteration_multiplier_override,
                    single_threaded: false,
                };
                return uct_search::search(battle, player, &config);
            }

            let Some(result) = search_v3::search(initial_battle, player, *budget_ms) else {
                debug!("StrategicV3 failed to initialize, falling back to MonteCarlo(50)");
                let battle = &player_state::randomize_battle_player(
                    initial_battle,
                    player.opponent(),
                    rand::rng().random(),
                );
                let config = UctConfig {
                    max_iterations_per_action: 50_000,
                    max_total_actions_multiplier: 6,
                    iteration_multiplier_override,
                    single_threaded: false,
                };
                return uct_search::search(battle, player, &config);
            };

            validated_strategic_action(
                initial_battle,
                player,
                result.action,
                result.pending_actions,
            )
        }
        GameAI::WaitFiveSeconds => {
            let battle = &player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            thread::sleep(Duration::from_secs(5));
            first_available_action(battle, player)
        }
    }
}

fn next_assignment_action(battle: &BattleState, player: PlayerName) -> Option<BattleAction> {
    PENDING_ASSIGNMENT.with(|cell| {
        let mut assignment = cell.borrow_mut();
        let a = (*assignment).as_ref()?;

        let legal = legal_actions::compute(battle, player);

        match &legal {
            LegalActions::SelectPositioningCharacter { .. } => {
                for &(char_id, placement) in &a.placements {
                    if let CharacterPlacement::MoveToFrontRank(_) = placement
                        && battle.cards.battlefield(player).is_in_back_rank(char_id)
                    {
                        return Some(BattleAction::SelectCharacterForPositioning(char_id));
                    }
                }
                *assignment = None;
                Some(BattleAction::EndTurn)
            }
            LegalActions::AssignColumn { character, .. } => {
                for &(char_id, placement) in &a.placements {
                    if char_id == *character
                        && let CharacterPlacement::MoveToFrontRank(col) = placement
                    {
                        return Some(BattleAction::MoveCharacterToFrontRank(char_id, col));
                    }
                }
                *assignment = None;
                None
            }
            _ => {
                *assignment = None;
                None
            }
        }
    })
}

fn next_strategic_action(battle: &BattleState, player: PlayerName) -> Option<BattleAction> {
    PENDING_STRATEGIC_ACTIONS.with(|cell| {
        let mut pending = cell.borrow_mut();
        let action = pending.front().copied()?;
        let legal = legal_actions::compute(battle, player);
        if legal.contains(action, ForPlayer::Agent) {
            pending.pop_front();
            Some(action)
        } else {
            pending.clear();
            None
        }
    })
}

fn validated_strategic_action(
    battle: &BattleState,
    player: PlayerName,
    action: BattleAction,
    pending_actions: Vec<BattleAction>,
) -> BattleAction {
    let legal = legal_actions::compute(battle, player);
    if legal.contains(action, ForPlayer::Agent) {
        PENDING_STRATEGIC_ACTIONS.with(|cell| {
            *cell.borrow_mut() = pending_actions.into_iter().collect();
        });
        action
    } else {
        PENDING_STRATEGIC_ACTIONS.with(|cell| {
            cell.borrow_mut().clear();
        });
        debug!(
            ?action,
            player = ?player,
            "Discarding stale strategic action and falling back to first available action"
        );
        first_available_action(battle, player)
    }
}

fn first_available_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player).all();
    *actions.first().unwrap()
}

fn random_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player).all();
    *actions.choose(&mut rand::rng()).unwrap()
}
