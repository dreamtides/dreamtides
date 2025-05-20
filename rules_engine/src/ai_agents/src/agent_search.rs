use std::time::Instant;

use ai_core::agent::{Agent, AgentConfig, AgentData};
use ai_data::game_ai::GameAI;
use ai_game_integration::evaluators::WinLossEvaluator;
use ai_game_integration::game_state_node_integration::AgentBattleState;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::iterative_deepening_search::IterativeDeepeningSearch;
use ai_uct::uct_config::UctConfig;
use ai_uct::{uct_search, uct_search_single_threaded};
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use rand::seq::IndexedRandom;
use tracing::{info, subscriber};
use tracing_macros::panic_with;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Selects an action for the given player using the given AI agent.
pub fn select_action(battle: &BattleState, player: PlayerName, game_ai: &GameAI) -> BattleAction {
    assert_eq!(legal_actions::next_to_act(battle), Some(player));

    let legal_actions = legal_actions::compute(battle, player);
    if legal_actions.is_empty() {
        panic_with!("No legal actions available for player", battle, player, legal_actions);
    }

    if legal_actions.len() == 1 {
        info!("Automatically selecting action {:?}", legal_actions.all()[0]);
        return legal_actions.all()[0];
    }

    let filter = EnvFilter::new(
        "warn,\
        ai_agents=debug,\
        ai_core=debug,\
        ai_data=debug,\
        ai_game_integration_old=debug,\
        ai_monte_carlo=debug,\
        ai_testing=debug,\
        ai_tree_search=debug,\
        ai_uct=debug",
    );
    let forest_subscriber =
        tracing_subscriber::registry().with(logging::create_forest_layer(filter));

    let start_time = Instant::now();
    let action = subscriber::with_default(forest_subscriber, || {
        select_action_unchecked(battle, player, game_ai)
    });
    info!(
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
) -> BattleAction {
    let battle = &player_state::randomize_battle_player(initial_battle, player.opponent());
    match game_ai {
        GameAI::AlwaysPanic => panic!("Always panic agent called for an action"),
        GameAI::FirstAvailableAction => first_available_action(battle, player),
        GameAI::RandomAction => random_action(battle, player),
        GameAI::IterativeDeepening => iterative_deepening_action(battle),
        GameAI::Uct1 => uct1_action(battle, 10, None),
        GameAI::Uct1MaxIterations(max_iterations) => {
            uct1_action(battle, 1000, Some(*max_iterations))
        }
        GameAI::NewUct(max_iterations) => {
            let config = UctConfig {
                max_iterations: *max_iterations,
                randomize_every_n_iterations: 100,
                ..Default::default()
            };
            uct_search_single_threaded::search_from_empty(battle, player, &config)
        }
        GameAI::ParallelUct(max_iterations) => {
            let config = UctConfig {
                max_iterations: *max_iterations,
                randomize_every_n_iterations: 100,
                ..Default::default()
            };
            uct_search::search_from_empty(battle, player, &config)
        }
        GameAI::ParallelUctRandomize(max_iterations) => {
            let config = UctConfig {
                max_iterations: *max_iterations,
                randomize_every_n_iterations: 1,
                ..Default::default()
            };
            uct_search::search_from_empty(battle, player, &config)
        }
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

fn iterative_deepening_action(battle: &BattleState) -> BattleAction {
    let agent =
        AgentData::omniscient("IterativeDeepening", IterativeDeepeningSearch, WinLossEvaluator);
    agent.pick_action(AgentConfig::with_deadline(1), &AgentBattleState { state: battle.clone() })
}

fn uct1_action(battle: &BattleState, deadline: u64, max_iterations: Option<u32>) -> BattleAction {
    let agent = AgentData::omniscient(
        "UCT1",
        MonteCarloAlgorithm { child_score_algorithm: Uct1 {}, max_iterations },
        RandomPlayoutEvaluator {},
    );
    agent.pick_action(AgentConfig::with_deadline(deadline), &AgentBattleState {
        state: battle.clone(),
    })
}
