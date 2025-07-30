#[derive(Debug, Clone)]
pub struct UctConfig {
    /// How many iterations of the monte carlo algorithm to run for each
    /// possible action under consideration.
    ///
    /// This value may be reduced if it would exceed max_total_iterations.
    pub max_iterations_per_action: u32,

    /// Maximum total number of iterations to run across all actions.
    ///
    /// If the number of available actions multiplied by
    /// max_iterations_per_action exceeds this value, the iterations per
    /// action will be reduced to stay within this cap.
    pub max_total_iterations: u32,

    /// Force all search logic onto one thread.
    ///
    /// Used for benchmarking.
    pub single_threaded: bool,
}
