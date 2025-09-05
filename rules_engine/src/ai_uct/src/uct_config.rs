#[derive(Debug, Clone)]
pub struct UctConfig {
    /// How many iterations of the monte carlo algorithm to run for each
    /// possible action under consideration.
    ///
    /// This value may be reduced when the number of available actions
    /// exceeds the configured multiplier budget (see
    /// max_total_actions_multiplier).
    pub max_iterations_per_action: u32,
    /// Multiplier determining the total iteration budget across all actions.
    ///
    /// Effective total iteration budget ~= max_iterations_per_action *
    /// max_total_actions_multiplier. If the number of legal actions exceeds
    /// this multiplier, iterations are divided proportionally so the total
    /// stays near the budget. If there are fewer actions, each gets the full
    /// max_iterations_per_action.
    pub max_total_actions_multiplier: u32,

    /// If set, overrides any dynamic iteration multipliers (prompt, phase,
    /// turn, energy-based) and applies this fixed multiplier instead.
    pub iteration_multiplier_override: Option<f64>,

    /// Force all search logic onto one thread.
    ///
    /// Used for benchmarking.
    pub single_threaded: bool,
}
