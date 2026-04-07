use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use ai_data::game_ai::GameAI;
use ai_uct::position_assignment::{CharacterPlacement, PositionAssignment};
use ai_uct::uct_config::UctConfig;
use ai_uct::{uct_search, uct_search_v2};
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use rand::Rng;
use rand::seq::IndexedRandom;
use tracing::{debug, instrument};

thread_local! {
    static PENDING_ASSIGNMENT: RefCell<Option<PositionAssignment>> = const { RefCell::new(None) };
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
    let battle = &player_state::randomize_battle_player(
        initial_battle,
        player.opponent(),
        rand::rng().random(),
    );
    match game_ai {
        GameAI::AlwaysPanic => panic!("Always panic agent called for an action"),
        GameAI::FirstAvailableAction => first_available_action(battle, player),
        GameAI::RandomAction => random_action(battle, player),
        GameAI::MonteCarlo(thousands_of_iterations) => {
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: false,
            };
            uct_search::search(battle, player, &config)
        }
        GameAI::MonteCarloSingleThreaded(thousands_of_iterations) => {
            let config = UctConfig {
                max_iterations_per_action: *thousands_of_iterations * 1000,
                max_total_actions_multiplier: 6,
                iteration_multiplier_override,
                single_threaded: true,
            };
            uct_search::search(battle, player, &config)
        }
        GameAI::MonteCarloV2(thousands_of_iterations) => {
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
        GameAI::WaitFiveSeconds => {
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

fn first_available_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player).all();
    *actions.first().unwrap()
}

fn random_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player).all();
    *actions.choose(&mut rand::rng()).unwrap()
}
