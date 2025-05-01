use action_data::battle_action_data::BattleAction;
use ai_core::agent::{Agent, AgentConfig, AgentData};
use ai_data::game_ai::GameAI;
use ai_game_integration::evaluators::WinLossEvaluator;
use ai_game_integration::state_node::AgentBattleState;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::iterative_deepening_search::IterativeDeepeningSearch;
use assert_with::panic_with;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;
use logging;
use rand::seq::IndexedRandom;
use tracing::subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Selects an action for the given player using the given AI agent.
pub fn select_action(battle: &BattleData, player: PlayerName, game_ai: &GameAI) -> BattleAction {
    assert_eq!(legal_actions::next_to_act(battle), Some(player));

    let legal_actions =
        legal_actions::compute(battle, player, LegalActions { for_human_player: false });
    if legal_actions.is_empty() {
        panic_with!(battle, "No legal actions available for player: {:?}", player);
    }
    if legal_actions.len() == 1 {
        return legal_actions[0];
    }

    let filter = EnvFilter::new(
        "warn,\
        ai_agents=debug,\
        ai_core=debug,\
        ai_data=debug,\
        ai_game_integration=debug,\
        ai_monte_carlo=debug,\
        ai_testing=debug,\
        ai_tree_search=debug,",
    );
    let forest_subscriber =
        tracing_subscriber::registry().with(logging::create_forest_layer(filter));
    subscriber::with_default(forest_subscriber, || match game_ai {
        GameAI::AlwaysPanic => panic!("Always panic agent called for an action"),
        GameAI::FirstAvailableAction => first_available_action(battle, player),
        GameAI::RandomAction => random_action(battle, player),
        GameAI::IterativeDeepening => iterative_deepening_action(battle),
        GameAI::Uct1 => uct1_action(battle, 10, None),
        GameAI::Uct1MaxIterations(max_iterations) => {
            uct1_action(battle, 1000, Some(*max_iterations))
        }
    })
}

fn first_available_action(battle: &BattleData, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player, LegalActions { for_human_player: false });
    *actions.first().unwrap()
}

fn random_action(battle: &BattleData, player: PlayerName) -> BattleAction {
    let actions = legal_actions::compute(battle, player, LegalActions { for_human_player: false });
    *actions.choose(&mut rand::rng()).unwrap()
}

fn iterative_deepening_action(battle: &BattleData) -> BattleAction {
    let agent =
        AgentData::omniscient("IterativeDeepening", IterativeDeepeningSearch, WinLossEvaluator);
    agent.pick_action(AgentConfig::with_deadline(1), &AgentBattleState(battle.clone()))
}

fn uct1_action(battle: &BattleData, deadline: u64, max_iterations: Option<u32>) -> BattleAction {
    let agent = AgentData::omniscient(
        "UCT1",
        MonteCarloAlgorithm { child_score_algorithm: Uct1 {}, max_iterations },
        RandomPlayoutEvaluator {},
    );
    agent.pick_action(AgentConfig::with_deadline(deadline), &AgentBattleState(battle.clone()))
}
