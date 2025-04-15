use std::iter;

use crate::game_state_node::GameStateNode;

/// StatePredictors address the problem of hidden information in games. Most
/// decision rules function effectively only for perfect-information games. The
/// agent system allows a StatePredictor to be defined which enumerates many
/// *possible* states which a game could currently be in given its actual
/// canonical game state.
///
/// The simplest StatePredictor is the [omniscient] predictor, which simply
/// returns the actual canonical game state with all hidden information
/// revealed. This is obviously the most effective approach in terms of AI
/// performance, but it is effectively cheating.
pub type StatePredictor<TNode> = fn(&TNode) -> Box<dyn Iterator<Item = TNode>>;

/// A [StatePredictor] which returns the actual canonical game state as the only
/// state prediction.
///
/// This creates an agent with perfect information about hidden game state, i.e.
/// one who cheats.
pub fn omniscient<N>(node: &N) -> Box<dyn Iterator<Item = N>>
where
    N: GameStateNode + 'static,
{
    Box::new(iter::once(node.make_copy()))
}
