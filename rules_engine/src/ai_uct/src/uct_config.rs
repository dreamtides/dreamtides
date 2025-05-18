#[derive(Debug, Clone)]
pub struct UctConfig {
    pub max_iterations: u32,

    /// How frequently to randomize the battle state being tested in order to
    /// consider different possible scenarios (opponent cards in hand etc).
    ///
    /// Lower values will slightly increase search time, but may cause the agent
    /// to play better.
    pub randomize_every_n_iterations: u32,

    /// Whether to save search tree data over the duration of the battle.
    pub persist_tree_between_searches: bool,

    /// Whether the agent has perfect knowledge of all hidden game information.
    ///
    /// Causes `randomize_every_n_iterations` to be ignored, since no
    /// randomization is ever required.
    pub omniscient: bool,
}

impl Default for UctConfig {
    fn default() -> Self {
        Self { max_iterations: 10_000, randomize_every_n_iterations: 100, persist_tree_between_searches: false, omniscient: false }
    }
}
