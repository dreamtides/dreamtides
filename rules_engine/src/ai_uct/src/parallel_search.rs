use std::collections::BTreeSet;

use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use petgraph::prelude::NodeIndex;

use crate::uct_config::UctConfig;
use crate::uct_search;
use crate::uct_tree::SearchGraph;

pub fn parallel_search(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
    graph: &mut SearchGraph,
    root: NodeIndex,
) -> BattleAction {
    uct_search::search(initial_battle, player, config, graph, root)
}

/// Returns an allocation of sets of legal actions from `legal_actions` to try
/// across `threads` different search threads.
///
/// This will always return a vector of length `threads` and attempt to evenly
/// distribute available legal actions to check between threads. In the event
/// that there are insufficent legal actions available to satisfy all available
/// threads, duplicate actions may be allocated to the remaining threads.
fn search_allocation(threads: usize, legal_actions: LegalActions) -> Vec<BTreeSet<BattleAction>> {
    let actions = legal_actions.all();
    let action_count = actions.len();

    if action_count == 0 {
        return vec![BTreeSet::new(); threads];
    }

    let mut result = vec![BTreeSet::new(); threads];
    let mut action_index = 0;

    for thread_index in 0..threads {
        let actions_per_thread = (action_count + threads - 1) / threads;
        for _ in 0..actions_per_thread {
            result[thread_index].insert(actions[action_index % action_count]);
            action_index += 1;
        }
    }

    result
}
