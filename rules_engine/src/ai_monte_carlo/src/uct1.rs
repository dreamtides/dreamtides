use std::f64::consts;

use crate::child_score::{ChildScoreAlgorithm, SelectionMode};

/// This implements the UCT1 algorithm for child scoring, a standard approach
/// for selecting children and solution to the 'multi-armed bandit' problem.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BESTCHILD(v,c)
///   𝐫𝐞𝐭𝐮𝐫𝐧 argmax(
///     v′ ∈ children of v:
///     Q(v′) / N(v′) +
///     c * √ [ 2 * ln(N(v)) / N(v′) ]
///   )
/// ```
pub struct Uct1 {}

impl ChildScoreAlgorithm for Uct1 {
    fn score(
        &self,
        parent_visits: f64,
        child_visits: f64,
        child_reward: f64,
        selection_mode: SelectionMode,
    ) -> f64 {
        let exploitation = child_reward / child_visits;
        let exploration = f64::sqrt((2.0 * f64::ln(parent_visits)) / child_visits);
        let exploration_bias = match selection_mode {
            SelectionMode::Exploration => consts::FRAC_1_SQRT_2,
            SelectionMode::Best => 0.0,
        };
        exploitation + (exploration_bias * exploration)
    }
}
