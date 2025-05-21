#[derive(Debug, Clone)]
pub struct UctConfig {
    /// How many iterations of the monte carlo algorithm to run for each
    /// possible action under consideration.
    ///
    /// Doing it this way (instead of a total max number of iterations) means
    /// the agent will 'think harder' when more actions are available and more
    /// effectively utilize a larger number of CPU threads. I think this is a
    /// good tradeoff.
    pub max_iterations_per_action: u32,
}
